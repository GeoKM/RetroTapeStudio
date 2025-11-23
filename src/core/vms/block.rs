use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum VmsBlockKind {
    Header,
    Continuation,
    FileData,
    Directory,
    Trailer,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct VmsBlock {
    pub index: u32,
    pub size: usize,

    /// The raw tape bytes (512 bytes)
    pub raw: Arc<[u8]>,

    /// 1–12 = BACKUP phase number
    pub phase: u16,

    /// Sequence number unique within a phase
    pub seq: u16,

    pub kind: VmsBlockKind,

    /// True if this block continues a previous file
    pub continuation: bool,
}

pub fn classify_vms_block(raw: &[u8], index: u32) -> Option<VmsBlock> {
    if raw.len() < 512 {
        return None;
    }

    // BACKUP structures always start with 0x01 0x00
    if raw[0] != 0x01 || raw[1] != 0x00 {
        return None;
    }

    let phase = u16::from_le_bytes([raw[2], raw[3]]);
    let seq = u16::from_le_bytes([raw[4], raw[5]]);
    let code = raw[6]; // block type code

    let kind = match code {
        0x01 => VmsBlockKind::Header,
        0x02 => VmsBlockKind::FileData,
        0x03 => VmsBlockKind::Continuation,
        0x04 => VmsBlockKind::Directory,
        0xFF => VmsBlockKind::Trailer,
        _ => VmsBlockKind::Unknown,
    };

    Some(VmsBlock {
        index,
        size: raw.len(),
        raw: Arc::from(raw.to_vec()),
        phase,
        seq,
        kind,
        continuation: code == 0x03,
    })
}
