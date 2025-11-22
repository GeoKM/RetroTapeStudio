use crate::core::block::TapeBlock;
use crate::core::parse::decode_rad50_word;

#[derive(Debug, Clone)]
pub struct RsxBlockInfo {
    pub name: String,
    pub uic: (u16, u16),
    pub protection: u16,
    pub is_directory: bool,
}

pub fn parse_block(block: &TapeBlock) -> Option<RsxBlockInfo> {
    let data = block.raw.as_ref();
    if data.len() < 16 {
        return None;
    }

    if data.get(0)? != &0x31 || data.get(1)? != &0x00 {
        return None;
    }

    let name_word1 = u16::from_le_bytes([data[2], data[3]]);
    let name_word2 = u16::from_le_bytes([data[4], data[5]]);
    let name_word3 = u16::from_le_bytes([data[6], data[7]]);
    let name = format!(
        "{}{}{}",
        decode_rad50_word(name_word1),
        decode_rad50_word(name_word2),
        decode_rad50_word(name_word3)
    );

    let status = u16::from_le_bytes([data[8], data[9]]);
    let uic = (
        u16::from_le_bytes([data[10], data[11]]),
        u16::from_le_bytes([data[12], data[13]]),
    );
    let protection = u16::from_le_bytes([data[14], data[15]]);
    let is_directory = (status & 0x8000) != 0;

    Some(RsxBlockInfo {
        name,
        uic,
        protection,
        is_directory,
    })
}
