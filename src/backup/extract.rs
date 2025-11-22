//! Assembles parsed VMS BACKUP blocks into file structures and directory trees for display and extraction.
use crate::backup::vms::{
    parse_directory_record, parse_fh2_record, parse_xh2_record, BackupBlock, VmsFileHeader,
};
use crate::tap::legacy::{TapDataKind, TapEntry};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmsFile {
    pub name: String,
    pub path: String,
    pub headers: VmsFileHeader,
    pub blocks: Vec<BackupBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VmsFileSystem {
    pub root: DirectoryNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirectoryNode {
    pub name: String,
    pub path: String,
    pub children: Vec<DirectoryNode>,
    pub files: Vec<String>,
}

/// Assemble VMS BACKUP Phase-1 blocks into files using sequence numbers.
///
/// A new file is started when a sequence number resets to 1 or stops increasing.
/// Blocks are grouped in the order they appear, and payload bytes are concatenated
/// in that same order.
pub fn assemble_files(entries: &[TapEntry]) -> Vec<ExtractedFile> {
    let vms_files = assemble_vms_files(entries);
    if !vms_files.is_empty() {
        return vms_files
            .into_iter()
            .enumerate()
            .map(|(idx, vf)| ExtractedFile {
                name: if vf.name.is_empty() {
                    format!("file_{}", idx)
                } else {
                    vf.name.clone()
                },
                blocks: vf.blocks,
            })
            .collect();
    }

    // Legacy fallback: group contiguous VMS blocks by sequence resets.
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

/// Build VmsFile objects from FH2/XH2 records and subsequent data blocks.
pub fn assemble_vms_files(entries: &[TapEntry]) -> Vec<VmsFile> {
    let mut files = Vec::new();
    let mut current_header: Option<VmsFileHeader> = None;
    let mut current_blocks: Vec<BackupBlock> = Vec::new();
    let mut current_dir = String::new();

    for block in entries.iter().filter_map(|entry| match &entry.kind {
        TapDataKind::VmsBlock(b) => Some(b.clone()),
        TapDataKind::Raw(_) => None,
    }) {
        if let Some(dir) = parse_directory_record(&block.payload) {
            current_dir = dir;
            continue;
        }
        if block.payload.first().copied() == Some(0x02) {
            // Finalize previous file if present.
            if let Some(header) = current_header.take() {
                let path = if current_dir.is_empty() {
                    header.full_name()
                } else {
                    format!("{}{}", current_dir, header.full_name())
                };
                files.push(VmsFile {
                    name: header.full_name(),
                    path,
                    headers: header,
                    blocks: current_blocks,
                });
                current_blocks = Vec::new();
            }
            if let Ok(mut header) = parse_fh2_record(&block.payload) {
                header.extended = None;
                current_header = Some(header);
            }
            continue;
        }

        if block.payload.first().copied() == Some(0x0C) {
            if let (Some(mut header), Ok(xh2)) =
                (current_header.take(), parse_xh2_record(&block.payload))
            {
                header.extended = Some(xh2);
                current_header = Some(header);
            }
            continue;
        }

        if let Some(_header) = current_header.as_ref() {
            current_blocks.push(block);
        }
    }

    if let Some(header) = current_header.take() {
        let path = if current_dir.is_empty() {
            header.full_name()
        } else {
            format!("{}{}", current_dir, header.full_name())
        };
        files.push(VmsFile {
            name: header.full_name(),
            path,
            headers: header,
            blocks: current_blocks,
        });
    }

    files
}

pub fn build_directory_tree(files: &[VmsFile]) -> VmsFileSystem {
    let mut root = DirectoryNode {
        name: "/".into(),
        path: "/".into(),
        children: Vec::new(),
        files: Vec::new(),
    };
    for file in files {
        insert_file(&mut root, &file.path, file.name.clone());
    }
    VmsFileSystem { root }
}

fn insert_file(root: &mut DirectoryNode, path: &str, name: String) {
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let mut node = root;
    let mut current_path = String::from("/");
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        if part.is_empty() {
            continue;
        }
        current_path.push_str(part);
        current_path.push('/');
        let idx = node
            .children
            .iter()
            .position(|c| c.name == *part)
            .unwrap_or_else(|| {
                node.children.push(DirectoryNode {
                    name: part.to_string(),
                    path: current_path.clone(),
                    children: Vec::new(),
                    files: Vec::new(),
                });
                node.children.len() - 1
            });
        node = node.children.get_mut(idx).unwrap();
    }
    node.files.push(name);
}

#[cfg(test)]
mod tests {
    use super::{assemble_files, assemble_vms_files, build_directory_tree, ExtractedFile, VmsFile};
    use crate::backup::vms::{BackupBlock, VmsFileHeader};
    use crate::tap::legacy::{TapDataKind, TapEntry};
    use crate::tap::DetectedFormat;

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
                detected_format: DetectedFormat::VmsBackup,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(2, b"world")),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
            // New file starts because sequence resets
            TapEntry {
                length: 11,
                kind: TapDataKind::VmsBlock(make_block(1, b"bye")),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
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
                detected_format: DetectedFormat::Raw,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(1, b"a")),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
        ];

        let files = assemble_files(&entries);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].payload(), b"a".to_vec());
    }

    fn fh2_payload(name: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(0x02);
        data.push(name.len() as u8);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&1u16.to_le_bytes()); // version
        data.push(0); // rfm
        data.extend_from_slice(&0x10u16.to_le_bytes()); // rattr
        data.extend_from_slice(&0x0001u16.to_le_bytes()); // protection
        data.extend_from_slice(&1u64.to_le_bytes()); // creation
        data.extend_from_slice(&2u64.to_le_bytes()); // revision
        data.extend_from_slice(&0u32.to_le_bytes()); // size high
        data.extend_from_slice(&1u32.to_le_bytes()); // size low
        data.extend_from_slice(&0x10u32.to_le_bytes()); // owner
        data.extend_from_slice(&0u16.to_le_bytes()); // rms attr
        data.extend_from_slice(&0u16.to_le_bytes()); // rms rattnr
        data.extend_from_slice(&0u16.to_le_bytes()); // rms rattnl
        data
    }

    #[test]
    fn assembles_vms_files_from_headers() {
        let entries = vec![
            TapEntry {
                length: 32,
                kind: TapDataKind::VmsBlock(make_block(1, &fh2_payload("DIR1.FILE1"))),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(2, b"data1")),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
            TapEntry {
                length: 32,
                kind: TapDataKind::VmsBlock(make_block(3, &fh2_payload("DIR1.FILE2"))),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
            TapEntry {
                length: 12,
                kind: TapDataKind::VmsBlock(make_block(4, b"data2")),
                log_level: None,
                detected_format: DetectedFormat::VmsBackup,
            },
        ];

        let files = assemble_vms_files(&entries);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].blocks.len(), 1);
        assert_eq!(files[1].blocks.len(), 1);
        assert!(files[0].path.contains("DIR1"));
    }

    #[test]
    fn builds_directory_tree_from_files() {
        let header = VmsFileHeader {
            file_name: "FILE1".into(),
            file_type: "TXT".into(),
            version: 1,
            record_format: crate::backup::vms::RecordFormat::Udf,
            record_attributes: 0,
            protection_mask: 0,
            creation_date: 0,
            revision_date: 0,
            file_size_high: 0,
            file_size_low: 0,
            owner_uic: 0,
            rms: crate::backup::vms::RmsAttributes {
                rfm: crate::backup::vms::RecordFormat::Udf,
                rattr: 0,
                rattnr: 0,
                rattnl: 0,
            },
            extended: None,
        };
        let file = VmsFile {
            name: "FILE1.TXT".into(),
            path: "/DIR/FILE1.TXT".into(),
            headers: header.clone(),
            blocks: vec![],
        };
        let fs = build_directory_tree(&[file]);
        assert_eq!(fs.root.children.len(), 1);
        assert_eq!(fs.root.children[0].files.len(), 1);
    }
}
