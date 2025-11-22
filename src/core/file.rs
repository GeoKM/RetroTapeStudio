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

// Placeholder metadata structs - filled in during Stage 6.
#[derive(Debug, Clone)]
pub struct VmsFileMetadata {
    pub placeholder: bool,
}

#[derive(Debug, Clone)]
pub struct RsxFileMetadata {
    pub placeholder: bool,
}

#[derive(Debug, Clone)]
pub struct Rt11FileMetadata {
    pub placeholder: bool,
}

#[derive(Debug, Clone)]
pub struct RstsFileMetadata {
    pub placeholder: bool,
}
