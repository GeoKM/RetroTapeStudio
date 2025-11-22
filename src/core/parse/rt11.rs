use crate::core::block::TapeBlock;
use crate::core::parse::decode_rad50_word;

#[derive(Debug, Clone)]
pub struct Rt11Entry {
    pub name: String,
    pub ext: String,
    pub start_block: u16,
    pub length_blocks: u16,
}

#[derive(Debug, Clone)]
pub struct Rt11BlockInfo {
    pub entries: Vec<Rt11Entry>,
}

pub fn parse_block(block: &TapeBlock) -> Option<Rt11BlockInfo> {
    let data = block.raw.as_ref();
    if data.len() != 512 {
        return None;
    }

    let mut entries = Vec::new();
    let mut offset = 0usize;

    while offset + 8 <= data.len() {
        let name_word = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let ext_word = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let start = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);
        let len = u16::from_le_bytes([data[offset + 6], data[offset + 7]]);

        if name_word == 0 && ext_word == 1 {
            break;
        }
        if name_word == 0 && ext_word == 0 && start == 0 && len == 0 {
            offset += 8;
            continue;
        }

        let entry = Rt11Entry {
            name: decode_rad50_word(name_word),
            ext: decode_rad50_word(ext_word),
            start_block: start,
            length_blocks: len,
        };
        entries.push(entry);

        offset += 8;
    }

    Some(Rt11BlockInfo { entries })
}
