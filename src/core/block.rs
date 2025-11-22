use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TapeBlock {
    pub index: u32,
    pub size: usize,
    pub raw: Arc<[u8]>,
    pub classification: BlockClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TapeFormat {
    Unknown,
    Vms,
    Rsx,
    Rt11,
    Rsts,
    Raw,
}

#[derive(Debug, Clone)]
pub enum BlockClassification {
    Unknown,
    Vms(VmsBlockKind),
    Rsx(RsxBlockKind),
    Rt11(Rt11BlockKind),
    Rsts(RstsBlockKind),
    Raw,
}

// For now, create empty placeholder enums.
// Codex will fill them in during Stage 5.
#[derive(Debug, Clone)]
pub enum VmsBlockKind {
    Placeholder,
}

#[derive(Debug, Clone)]
pub enum RsxBlockKind {
    Placeholder,
}

#[derive(Debug, Clone)]
pub enum Rt11BlockKind {
    Placeholder,
}

#[derive(Debug, Clone)]
pub enum RstsBlockKind {
    Placeholder,
}
