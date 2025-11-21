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
    let (kind, mut detected_format) = if length > 20 {
        match read_backup_block(record) {
            Ok(block) => (TapDataKind::VmsBlock(block), DetectedFormat::VmsBackup),
            Err(_) => (TapDataKind::Raw(record.to_vec()), DetectedFormat::Raw),
        }
    } else {
        (TapDataKind::Raw(record.to_vec()), DetectedFormat::Raw)
    };

    if detected_format == DetectedFormat::Raw {
        if detect_rsx11m(record) {
            detected_format = DetectedFormat::Rsx11m;
        } else if detect_rt11(record) {
            detected_format = DetectedFormat::Rt11;
        } else if detect_rsts(record) {
            detected_format = DetectedFormat::RstsE;
        }
    }

    Ok(TapEntry {
        length,
        kind,
        log_level: None,
        detected_format,
    })
}

fn detect_rsx11m(block: &[u8]) -> bool {
    if block.len() < 512 || block.len() % 512 != 0 {
        return false;
    }
    let has_count = block.get(0..4) == Some(&[0x01, 0x00, 0x00, 0x00]);
    let has_tag = block.windows(3).any(|w| w == b"RSX" || w == b"UFD");
    has_count && has_tag
}

fn detect_rt11(block: &[u8]) -> bool {
    if block.len() < 512 || block.len() % 512 != 0 {
        return false;
    }
    let dir_words = u16::from_le_bytes([block[0], block[1]]);
    let plausible_dir = dir_words > 0 && dir_words < 0x0400;
    let has_tag = block.windows(4).any(|w| w == b"RT11");
    plausible_dir && has_tag
}

fn detect_rsts(block: &[u8]) -> bool {
    if block.len() < 512 || block.len() % 512 != 0 {
        return false;
    }
    if block.len() < 64 {
        return false;
    }
    let first = &block[0..32];
    let repeated = &block[32..64];
    let bitmap_like = first == repeated;
    let has_tag = block.windows(4).any(|w| w == b"RSTS");
    bitmap_like && has_tag
}

#[cfg(test)]
mod tests {
    use super::{detect_rsts, detect_rsx11m, detect_rt11, read_tap_entry};
    use crate::tap::reader::TapDataKind;
    use crate::tap::DetectedFormat;
    use crate::TapeError;
    use std::fs;

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

    #[test]
    fn test_detect_rt11() {
        let data = fs::read("tests/data/rt11.tap").expect("missing rt11 fixture");
        assert!(detect_rt11(&data));
    }

    #[test]
    fn test_detect_rsx11m() {
        let data = fs::read("tests/data/rsx.tap").expect("missing rsx fixture");
        assert!(detect_rsx11m(&data));
    }

    #[test]
    fn test_detect_rsts() {
        let data = fs::read("tests/data/rsts.tap").expect("missing rsts fixture");
        assert!(detect_rsts(&data));
    }
}
