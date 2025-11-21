use crate::backup::vms::BackupBlock;
use crate::tap::reader::{TapDataKind, TapEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    pub name: String,
    pub blocks: Vec<BackupBlock>,
}

impl ExtractedFile {
    /// Concatenate payload bytes from all blocks in order.
    pub fn payload(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for block in &self.blocks {
            data.extend_from_slice(&block.payload);
        }
        data
    }
}

/// Assemble VMS BACKUP Phase-1 blocks into files using sequence numbers.
///
/// A new file is started when a sequence number resets to 1 or stops increasing.
/// Blocks are grouped in the order they appear, and payload bytes are concatenated
/// in that same order.
pub fn assemble_files(entries: &[TapEntry]) -> Vec<ExtractedFile> {
    let mut files = Vec::new();
    let mut current_blocks: Vec<BackupBlock> = Vec::new();
    let mut file_index = 0usize;

    for block in entries.iter().filter_map(|entry| match &entry.kind {
        TapDataKind::VmsBlock(b) => Some(b.clone()),
        TapDataKind::Raw(_) => None,
    }) {
        let start_new = current_blocks
            .last()
            .map(|last| block.sequence_number <= last.sequence_number || block.sequence_number == 1)
            .unwrap_or(false);

        if start_new && !current_blocks.is_empty() {
            let name = format!("file_{}", file_index);
            files.push(ExtractedFile {
                name,
                blocks: current_blocks,
            });
            file_index += 1;
            current_blocks = Vec::new();
        }

        current_blocks.push(block);
    }

    if !current_blocks.is_empty() {
        let name = format!("file_{}", file_index);
        files.push(ExtractedFile {
            name,
            blocks: current_blocks,
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::{assemble_files, ExtractedFile};
    use crate::backup::vms::BackupBlock;
    use crate::tap::reader::{TapDataKind, TapEntry};

    fn make_block(seq: u32, payload: &[u8]) -> BackupBlock {
        BackupBlock {
            block_size: (10 + payload.len()) as u16,
            format_version: 1,
            phase: 1,
            sequence_number: seq,
            checksum: 0,
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn groups_blocks_by_sequence() {
        let entries = vec![
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(1, b"hello ")),
                log_level: None,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(2, b"world")),
                log_level: None,
            },
            // New file starts because sequence resets
            TapEntry {
                length: 11,
                kind: TapDataKind::VmsBlock(make_block(1, b"bye")),
                log_level: None,
            },
        ];

        let files = assemble_files(&entries);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].name, "file_0");
        assert_eq!(files[1].name, "file_1");

        let payloads: Vec<Vec<u8>> = files.iter().map(ExtractedFile::payload).collect();
        assert_eq!(payloads[0], b"hello world".to_vec());
        assert_eq!(payloads[1], b"bye".to_vec());
    }

    #[test]
    fn ignores_raw_entries() {
        let entries = vec![
            TapEntry {
                length: 5,
                kind: TapDataKind::Raw(vec![1, 2, 3]),
                log_level: None,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(1, b"a")),
                log_level: None,
            },
        ];

        let files = assemble_files(&entries);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].payload(), b"a".to_vec());
    }
}
