use crate::backup::vms::{read_backup_block, BackupBlock};
use crate::log::parse::LogLevel;
use crate::tap::DetectedFormat;
use crate::{TapeError, TapeResult};

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
    pub detected_format: DetectedFormat,
}

/// Parse a TAP record from already isolated record bytes.
///
/// If the record is larger than 20 bytes, a best-effort VMS BACKUP Phase-1
/// decode is attempted; on success, the parsed block is returned. Otherwise,
/// the raw bytes are surfaced.
pub fn read_tap_entry(record: &[u8]) -> TapeResult<TapEntry> {
    if record.is_empty() {
        return Err(TapeError::Parse("empty TAP record".into()));
    }
    if record.len() > 1_048_576 {
        return Err(TapeError::UnsupportedFormat("record too large".into()));
    }

    let length = record.len();
    let (kind, detected_format) = if length > 20 {
        match read_backup_block(record) {
            Ok(block) => (TapDataKind::VmsBlock(block), DetectedFormat::VmsBackup),
            Err(err) => return Err(err),
        }
    } else {
        (TapDataKind::Raw(record.to_vec()), DetectedFormat::Raw)
    };

    Ok(TapEntry {
        length,
        kind,
        log_level: None,
        detected_format,
    })
}

#[cfg(test)]
mod tests {
    use super::read_tap_entry;
    use crate::tap::reader::TapDataKind;
    use crate::tap::DetectedFormat;
    use crate::TapeError;

    #[test]
    fn keeps_small_records_raw() {
        let record = vec![1, 2, 3];
        let entry = read_tap_entry(&record).expect("should parse");
        assert_eq!(entry.length, 3);
        assert_eq!(entry.detected_format, DetectedFormat::Raw);
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
        assert_eq!(entry.detected_format, DetectedFormat::VmsBackup);
    }

    #[test]
    fn rejects_empty_record() {
        let err = read_tap_entry(&[]).unwrap_err();
        match err {
            TapeError::Parse(_) => {}
            _ => panic!("expected parse error"),
        }
    }
}
