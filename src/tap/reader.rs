use crate::backup::vms::{read_backup_block, BackupBlock};
use crate::log::parse::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapDataKind {
    Raw(Vec<u8>),
    VmsBlock(BackupBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapEntry {
    pub length: usize,
    pub kind: TapDataKind,
    pub log_level: Option<LogLevel>,
}

/// Parse a TAP record from already isolated record bytes.
///
/// If the record is larger than 20 bytes, a best-effort VMS BACKUP Phase-1
/// decode is attempted; on success, the parsed block is returned. Otherwise,
/// the raw bytes are surfaced.
pub fn read_tap_entry(record: &[u8]) -> Result<TapEntry, String> {
    if record.is_empty() {
        return Err("empty TAP record".into());
    }

    let length = record.len();
    let kind = if length > 20 {
        match read_backup_block(record) {
            Ok(block) => TapDataKind::VmsBlock(block),
            Err(_) => TapDataKind::Raw(record.to_vec()),
        }
    } else {
        TapDataKind::Raw(record.to_vec())
    };

    Ok(TapEntry {
        length,
        kind,
        log_level: None,
    })
}

#[cfg(test)]
mod tests {
    use super::read_tap_entry;
    use crate::tap::reader::TapDataKind;

    #[test]
    fn keeps_small_records_raw() {
        let record = vec![1, 2, 3];
        let entry = read_tap_entry(&record).expect("should parse");
        assert_eq!(entry.length, 3);
        match entry.kind {
            TapDataKind::Raw(data) => assert_eq!(data, record),
            _ => panic!("expected raw data"),
        }
    }

    #[test]
    fn detects_vms_block() {
        let mut raw = vec![0u8; 32];
        raw[0..2].copy_from_slice(&32u16.to_le_bytes());
        raw[2] = 2;
        raw[3] = 1;
        raw[4..8].copy_from_slice(&9u32.to_le_bytes());
        raw[8..10].copy_from_slice(&0x1234u16.to_le_bytes());
        raw[10] = 0xAA;

        let entry = read_tap_entry(&raw).expect("should parse");
        match entry.kind {
            TapDataKind::VmsBlock(block) => {
                assert_eq!(block.block_size as usize, 32);
                assert_eq!(block.payload[0], 0xAA);
            }
            _ => panic!("expected VMS block"),
        }
    }
}
