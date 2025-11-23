use std::sync::Arc;

/// Implements parsing of FH2, XH2, XH3, and VBN blocks found in VMS Backup savesets.
/// The caller (detector) already identified that this block belongs to VMS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmsBlockKind {
    FileHeader(Fh2Record),
    Extension(Xh2Record),
    Ident(Xh3Record),
    FileData(VbnRecord),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fh2Record {
    pub file_name: String,
    pub file_type: String,
    pub version: u16,
    pub uic: (u16, u16),
    pub record_format: u8,
    pub record_attributes: u8,
    pub block_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xh2Record {
    pub backup_flags: u16,
    pub journal_flags: u16,
    pub high_precision_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xh3Record {
    pub file_id: u32,
    pub sequence_number: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VbnRecord {
    pub vbn: u32,
    pub payload: Arc<[u8]>,
}

#[derive(Debug, Clone)]
pub struct VmsBlock {
    pub index: u32,
    pub raw: Arc<[u8]>,
    pub kind: VmsBlockKind,
}

pub fn classify_vms_block(raw: &[u8], index: u32) -> Option<VmsBlock> {
    if raw.len() < 8 {
        return None;
    }
    let raw_arc: Arc<[u8]> = Arc::from(raw.to_vec());
    let kind = parse_vms_block(Arc::clone(&raw_arc));
    if matches!(kind, VmsBlockKind::Unknown) {
        return None;
    }
    Some(VmsBlock {
        index,
        raw: raw_arc,
        kind,
    })
}

pub fn parse_vms_block(raw: Arc<[u8]>) -> VmsBlockKind {
    if raw.len() < 8 {
        return VmsBlockKind::Unknown;
    }

    let code = raw[0];
    match code {
        0xC0 => parse_fh2(raw),
        0xC1 => parse_xh2(raw),
        0xC2 => parse_xh3(raw),
        0xC4 => parse_vbn(raw),
        _ => VmsBlockKind::Unknown,
    }
}

fn parse_fh2(raw: Arc<[u8]>) -> VmsBlockKind {
    if raw.len() < 32 {
        return VmsBlockKind::Unknown;
    }

    let name_len = raw[8] as usize;
    let type_len = raw[9] as usize;

    if 10 + name_len + type_len > raw.len() {
        return VmsBlockKind::Unknown;
    }

    let name = String::from_utf8_lossy(&raw[10..10 + name_len]).to_string();
    let ftype = String::from_utf8_lossy(&raw[10 + name_len..10 + name_len + type_len]).to_string();

    let version = u16::from_le_bytes([raw[4], raw[5]]);
    let uic0 = u16::from_le_bytes([raw[12], raw[13]]);
    let uic1 = u16::from_le_bytes([raw[14], raw[15]]);

    let rfm = raw[16];
    let rat = raw[17];

    let block_count = u32::from_le_bytes([raw[20], raw[21], raw[22], raw[23]]);

    VmsBlockKind::FileHeader(Fh2Record {
        file_name: name,
        file_type: ftype,
        version,
        uic: (uic0, uic1),
        record_format: rfm,
        record_attributes: rat,
        block_count,
    })
}

fn parse_xh2(raw: Arc<[u8]>) -> VmsBlockKind {
    if raw.len() < 16 {
        return VmsBlockKind::Unknown;
    }
    let flags = u16::from_le_bytes([raw[4], raw[5]]);
    let journal = u16::from_le_bytes([raw[6], raw[7]]);
    let hptime = u64::from_le_bytes([
        raw[8], raw[9], raw[10], raw[11], raw[12], raw[13], raw[14], raw[15],
    ]);

    VmsBlockKind::Extension(Xh2Record {
        backup_flags: flags,
        journal_flags: journal,
        high_precision_time: hptime,
    })
}

fn parse_xh3(raw: Arc<[u8]>) -> VmsBlockKind {
    if raw.len() < 12 {
        return VmsBlockKind::Unknown;
    }

    let file_id = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
    let seq = u16::from_le_bytes([raw[8], raw[9]]);

    VmsBlockKind::Ident(Xh3Record {
        file_id,
        sequence_number: seq,
    })
}

fn parse_vbn(raw: Arc<[u8]>) -> VmsBlockKind {
    if raw.len() < 8 {
        return VmsBlockKind::Unknown;
    }
    let vbn = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);

    let payload = raw[8..].to_vec().into();

    VmsBlockKind::FileData(VbnRecord { vbn, payload })
}
