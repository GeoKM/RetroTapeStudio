use super::*;

#[derive(Debug, Clone)]
pub struct VmsReconstructedFile {
    pub header: VmsFileHeader,
    pub blocks: Vec<u32>,
    pub data: Vec<u8>,
}
