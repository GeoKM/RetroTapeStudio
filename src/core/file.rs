use super::block::TapeFormat;

#[derive(Debug, Clone)]
pub struct TapePath {
    pub elements: Vec<String>,
}

impl TapePath {
    pub fn new(elements: Vec<String>) -> Self {
        Self { elements }
    }

    pub fn to_string_path(&self) -> String {
        self.elements.join("/")
    }
}

#[derive(Debug, Clone)]
pub struct TapeFile {
    pub format: TapeFormat,
    pub path: TapePath,
    pub size_bytes: u64,
    pub blocks: Vec<u32>,
    pub metadata: FileMetadata,
    pub children: Vec<TapeFile>,
}

#[derive(Debug, Clone)]
pub enum FileMetadata {
    Vms(VmsFileMetadata),
    Rsx(RsxFileMetadata),
    Rt11(Rt11FileMetadata),
    Rsts(RstsFileMetadata),
    Raw,
}

/// Fully decoded VMS header metadata (FH2 + optional XH2)
#[derive(Debug, Clone)]
pub struct VmsFileMetadata {
    pub file_id: (u16, u16, u16),
    pub rev: u16,
    pub seq: u16,
    pub owner_uic: (u16, u16),
    pub protection: u16,
    pub record_format: u8,
    pub record_attributes: u8,
    pub record_length: u16,
    pub file_type: String,
    pub backup_flags: u16,
    pub creation_time: Option<String>,
    pub revision_time: Option<String>,
    pub expiration_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RsxFileMetadata {
    pub uic: (u16, u16),
    pub protection: u16,
    pub is_directory: bool,
}

#[derive(Debug, Clone)]
pub struct Rt11FileMetadata {
    pub start_block: u16,
    pub length_blocks: u16,
    pub ext: String,
}

#[derive(Debug, Clone)]
pub struct RstsFileMetadata {
    pub owner_uic: (u16, u16),
    pub blocks: u16,
    pub status: u16,
}
