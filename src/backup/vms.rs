use std::convert::TryInto;

use crate::{TapeError, TapeResult};

/// Parsed view of a VMS BACKUP Phase-1 block header and payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBlock {
    pub block_size: u16,
    pub format_version: u8,
    pub phase: u8,
    pub sequence_number: u32,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordFormat {
    Udf,
    Vfc,
    Var,
    Fix,
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RmsAttributes {
    pub rfm: RecordFormat,
    pub rattr: u16,
    pub rattnr: u16,
    pub rattnl: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VmsExtendedHeader {
    pub backup_flags: Option<u16>,
    pub high_precision_timestamp: Option<u64>,
    pub acp_attributes: Option<u32>,
    pub journaling_flags: Option<u16>,
    pub file_id: Option<u32>,
    pub sequence_number: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmsFileHeader {
    pub file_name: String,
    pub file_type: String,
    pub version: u16,
    pub record_format: RecordFormat,
    pub record_attributes: u16,
    pub protection_mask: u16,
    pub creation_date: u64,
    pub revision_date: u64,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub owner_uic: u32,
    pub rms: RmsAttributes,
    pub extended: Option<VmsExtendedHeader>,
}

impl VmsFileHeader {
    pub fn full_name(&self) -> String {
        if self.file_type.is_empty() {
            self.file_name.clone()
        } else {
            format!("{}.{}", self.file_name, self.file_type)
        }
    }
}

/// Read a Phase-1 VMS BACKUP block from raw bytes.
///
/// The expected layout is:
/// - 0..2: little-endian block size (bytes within this block).
/// - 2: format version byte.
/// - 3: phase identifier (must be 1 for Phase-1 blocks).
/// - 4..8: little-endian sequence number.
/// - 8..10: little-endian checksum field (exposed but not validated here).
/// - 10..N: payload bytes recorded by BACKUP.
pub fn read_backup_block(data: &[u8]) -> TapeResult<BackupBlock> {
    const HEADER_LEN: usize = 10;

    if data.len() < HEADER_LEN {
        return Err(TapeError::Parse("buffer too small for VMS BACKUP header".into()));
    }

    let block_size = u16::from_le_bytes([data[0], data[1]]);
    if block_size == 0 {
        return Err(TapeError::Parse("block size must be greater than zero".into()));
    }

    let block_size_usize = block_size as usize;
    if block_size_usize > data.len() {
        return Err(TapeError::Parse(format!(
            "block size {} exceeds available data {}",
            block_size,
            data.len()
        )));
    }

    let format_version = data[2];
    let phase = data[3];
    if phase != 1 {
        return Err(TapeError::UnsupportedFormat(format!(
            "unsupported phase {}, expected Phase-1",
            phase
        )));
    }

    let sequence_number = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let checksum = u16::from_le_bytes([data[8], data[9]]);

    let payload = data[HEADER_LEN..block_size_usize].to_vec();

    Ok(BackupBlock {
        block_size,
        format_version,
        phase,
        sequence_number,
        checksum,
        payload,
    })
}

const FH2_CODE: u8 = 0x02;
const XH2_CODE: u8 = 0x0C;
const DIR_CODE: u8 = 0x04;

pub fn parse_record_format(code: u8) -> RecordFormat {
    match code {
        0 => RecordFormat::Udf,
        1 => RecordFormat::Vfc,
        2 => RecordFormat::Var,
        3 => RecordFormat::Fix,
        other => RecordFormat::Unknown(other),
    }
}

/// Parse a VMS BACKUP FH2 header record.
///
/// Layout (all little-endian unless noted):
/// 0: record code (0x02)
/// 1: name length (u8)
/// 2..: file name bytes (ASCII)
/// after name: u16 version
/// next: u8 record format code
/// next: u16 record attributes
/// next: u16 protection mask
/// next: u64 creation date
/// next: u64 revision date
/// next: u32 file size high
/// next: u32 file size low
/// next: u32 owner UIC
/// next: u16 rms rattr
/// next: u16 rms rattnr
/// next: u16 rms rattnl
pub fn parse_fh2_record(data: &[u8]) -> TapeResult<VmsFileHeader> {
    if data.is_empty() || data[0] != FH2_CODE {
        return Err(TapeError::Parse("FH2 record code missing".into()));
    }
    if data.len() < 1 + 1 + 2 + 1 + 2 + 2 + 8 + 8 + 4 + 4 + 4 + 2 + 2 + 2 {
        return Err(TapeError::Parse("FH2 record too short".into()));
    }

    let name_len = data[1] as usize;
    if data.len() < 2 + name_len + 2 + 1 + 2 + 2 + 8 + 8 + 4 + 4 + 4 + 2 + 2 + 2 {
        return Err(TapeError::Parse("FH2 record missing name bytes".into()));
    }

    let name_bytes = &data[2..2 + name_len];
    let name_str = String::from_utf8_lossy(name_bytes).to_string();
    let mut offset = 2 + name_len;

    let version = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    offset += 2;

    let record_format = parse_record_format(data[offset]);
    offset += 1;

    let record_attributes = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    offset += 2;

    let protection_mask = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    offset += 2;

    let creation_date = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;
    let revision_date = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    let file_size_high = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;
    let file_size_low = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let owner_uic = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let rattr = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    offset += 2;
    let rattnr = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    offset += 2;
    let rattnl = u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
    let rms = RmsAttributes {
        rfm: record_format.clone(),
        rattr,
        rattnr,
        rattnl,
    };

    let (file_name, file_type) = split_name_type(&name_str);

    Ok(VmsFileHeader {
        file_name,
        file_type,
        version,
        record_format,
        record_attributes,
        protection_mask,
        creation_date,
        revision_date,
        file_size_high,
        file_size_low,
        owner_uic,
        rms,
        extended: None,
    })
}

fn split_name_type(name: &str) -> (String, String) {
    match name.rsplit_once('.') {
        Some((n, t)) => (n.to_string(), t.to_string()),
        None => (name.to_string(), String::new()),
    }
}

/// Parse an XH2 extended header record.
pub fn parse_xh2_record(data: &[u8]) -> TapeResult<VmsExtendedHeader> {
    if data.is_empty() || data[0] != XH2_CODE {
        return Err(TapeError::Parse("XH2 record code missing".into()));
    }
    if data.len() < 1 + 2 + 8 + 4 + 2 + 4 + 4 {
        return Err(TapeError::Parse("XH2 record too short".into()));
    }

    let mut offset = 1;
    let backup_flags = Some(u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap()));
    offset += 2;
    let high_precision_timestamp =
        Some(u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()));
    offset += 8;
    let acp_attributes = Some(u32::from_le_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ));
    offset += 4;
    let journaling_flags =
        Some(u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap()));
    offset += 2;
    let file_id = Some(u32::from_le_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ));
    offset += 4;
    let sequence_number = Some(u32::from_le_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ));
    let header = VmsExtendedHeader {
        backup_flags,
        high_precision_timestamp,
        acp_attributes,
        journaling_flags,
        file_id,
        sequence_number,
    };
    Ok(header)
}

/// Parse a directory record payload, returning the path if present.
pub fn parse_directory_record(data: &[u8]) -> Option<String> {
    if data.first().copied()? != DIR_CODE {
        return None;
    }
    if data.len() < 2 {
        return None;
    }
    let len = data[1] as usize;
    if data.len() < 2 + len {
        return None;
    }
    let path = String::from_utf8_lossy(&data[2..2 + len]).to_string();
    Some(path)
}

/// Format a VMS protection mask into a four-field string.
pub fn format_protection(mask: u16) -> String {
    let classes = ["System", "Owner", "Group", "World"];
    let mut parts = Vec::new();
    for (i, _name) in classes.iter().enumerate() {
        let shift = (3 - i) * 4;
        let nibble = ((mask >> shift) & 0xF) as u8;
        parts.push(format!("({})", format_rights(nibble)));
    }
    parts.join(",")
}

fn format_rights(bits: u8) -> String {
    let mut rights = String::new();
    if bits & 0x8 != 0 {
        rights.push('R');
    }
    if bits & 0x4 != 0 {
        rights.push('W');
    }
    if bits & 0x2 != 0 {
        rights.push('E');
    }
    if bits & 0x1 != 0 {
        rights.push('D');
    }
    rights
}

#[cfg(test)]
mod tests {
    use super::{
        format_protection, parse_directory_record, parse_fh2_record, parse_record_format,
        parse_xh2_record, read_backup_block, RecordFormat,
    };
    use crate::TapeError;

    #[test]
    fn parses_minimal_block() {
        let mut raw = vec![0u8; 12];
        raw[0..2].copy_from_slice(&12u16.to_le_bytes());
        raw[2] = 2; // format version
        raw[3] = 1; // phase
        raw[4..8].copy_from_slice(&5u32.to_le_bytes());
        raw[8..10].copy_from_slice(&0xABCDu16.to_le_bytes());
        raw[10] = 0x11;
        raw[11] = 0x22;

        let block = read_backup_block(&raw).expect("should parse");

        assert_eq!(block.block_size, 12);
        assert_eq!(block.format_version, 2);
        assert_eq!(block.phase, 1);
        assert_eq!(block.sequence_number, 5);
        assert_eq!(block.checksum, 0xABCD);
        assert_eq!(block.payload, vec![0x11, 0x22]);
    }

    #[test]
    fn rejects_non_phase_one() {
        let mut raw = vec![0u8; 10];
        raw[0..2].copy_from_slice(&10u16.to_le_bytes());
        raw[3] = 2;

        let err = read_backup_block(&raw).unwrap_err();
        match err {
            TapeError::UnsupportedFormat(msg) => assert!(msg.contains("Phase-1")),
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn parses_fh2_record() {
        let mut data = Vec::new();
        data.push(0x02);
        let name = "FILE.TXT";
        data.push(name.len() as u8);
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&3u16.to_le_bytes()); // version
        data.push(2); // record format VAR
        data.extend_from_slice(&0x10u16.to_le_bytes()); // rattr
        data.extend_from_slice(&0x1234u16.to_le_bytes()); // protection
        data.extend_from_slice(&11u64.to_le_bytes()); // creation
        data.extend_from_slice(&22u64.to_le_bytes()); // revision
        data.extend_from_slice(&1u32.to_le_bytes()); // size high
        data.extend_from_slice(&2u32.to_le_bytes()); // size low
        data.extend_from_slice(&0xAAAAu32.to_le_bytes()); // owner
        data.extend_from_slice(&0x20u16.to_le_bytes()); // rms rattr
        data.extend_from_slice(&0x30u16.to_le_bytes()); // rms rattnr
        data.extend_from_slice(&0x40u16.to_le_bytes()); // rms rattnl

        let header = parse_fh2_record(&data).unwrap();
        assert_eq!(header.file_name, "FILE");
        assert_eq!(header.file_type, "TXT");
        assert_eq!(header.version, 3);
        assert!(matches!(header.record_format, RecordFormat::Var));
        assert_eq!(header.record_attributes, 0x10);
        assert_eq!(header.protection_mask, 0x1234);
        assert_eq!(header.creation_date, 11);
        assert_eq!(header.revision_date, 22);
        assert_eq!(header.file_size_high, 1);
        assert_eq!(header.file_size_low, 2);
        assert_eq!(header.owner_uic, 0xAAAA);
        assert_eq!(header.rms.rattr, 0x20);
        assert_eq!(header.rms.rattnr, 0x30);
        assert_eq!(header.rms.rattnl, 0x40);
    }

    #[test]
    fn parses_xh2_record() {
        let mut data = Vec::new();
        data.push(0x0C);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&2u64.to_le_bytes());
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(&4u16.to_le_bytes());
        data.extend_from_slice(&5u32.to_le_bytes());
        data.extend_from_slice(&6u32.to_le_bytes());

        let xh2 = parse_xh2_record(&data).unwrap();
        assert_eq!(xh2.backup_flags, Some(1));
        assert_eq!(xh2.high_precision_timestamp, Some(2));
        assert_eq!(xh2.acp_attributes, Some(3));
        assert_eq!(xh2.journaling_flags, Some(4));
        assert_eq!(xh2.file_id, Some(5));
        assert_eq!(xh2.sequence_number, Some(6));
    }

    #[test]
    fn parses_directory_record() {
        let mut data = vec![0x04, 5];
        data.extend_from_slice(b"DIR1/");
        let path = parse_directory_record(&data).unwrap();
        assert_eq!(path, "DIR1/");
    }

    #[test]
    fn formats_protection() {
        let text = format_protection(0xFFFF);
        assert!(text.contains("R") && text.contains("W") && text.contains("E") && text.contains("D"));
        let parts: Vec<&str> = text.split(',').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn maps_record_format() {
        assert!(matches!(parse_record_format(0), RecordFormat::Udf));
        assert!(matches!(parse_record_format(3), RecordFormat::Fix));
        assert!(matches!(parse_record_format(9), RecordFormat::Unknown(_)));
    }
}
