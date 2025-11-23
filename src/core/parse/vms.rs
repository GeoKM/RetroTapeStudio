//! VMS FH2/XH2 metadata parser.

use crate::core::block::TapeBlock;
use crate::core::file::VmsFileMetadata;

/// Convert a VMS 64-bit timestamp (100ns ticks since 1858-11-17)
/// into ISO8601. If zero -> None.
pub fn decode_vms_time(qw: u64) -> Option<String> {
    if qw == 0 {
        return None;
    }
    // Convert 100ns ticks to seconds
    let secs = qw / 10_000_000;
    // VMS epoch -> Unix epoch offset
    const VMS_TO_UNIX: i64 = -3_506_716_800; // seconds
    let unix = secs as i64 + VMS_TO_UNIX;
    if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(unix, 0) {
        Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    }
}

/// Extract FH2 fields from a VMS header block.
pub fn parse_vms_fh2(block: &TapeBlock) -> Option<VmsFileMetadata> {
    let d = block.raw.as_ref();
    if d.len() < 64 {
        return None;
    }

    // FH2 ID must be 0x02,0x00
    if d.get(0) != Some(&0x02) || d.get(1) != Some(&0x00) {
        return None;
    }

    let u16le = |off: usize| u16::from_le_bytes([d[off], d[off + 1]]);
    let u64le = |off: usize| {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&d[off..off + 8]);
        u64::from_le_bytes(bytes)
    };

    Some(VmsFileMetadata {
        file_id: (u16le(12), u16le(14), u16le(16)),
        rev: u16le(18),
        seq: u16le(20),
        owner_uic: (u16le(22), u16le(24)),
        protection: u16le(26),
        record_format: d[28],
        record_attributes: d[29],
        record_length: u16le(30),
        file_type: decode_filetype(u16le(32)),
        backup_flags: u16le(34),
        creation_time: decode_vms_time(u64le(40)),
        revision_time: decode_vms_time(u64le(48)),
        expiration_time: decode_vms_time(u64le(56)),
    })
}

fn decode_filetype(code: u16) -> String {
    match code {
        1 => "Sequential".to_string(),
        2 => "Relative".to_string(),
        3 => "Indexed".to_string(),
        _ => format!("Unknown({code})"),
    }
}
