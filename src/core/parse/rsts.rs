use crate::core::block::TapeBlock;
use crate::core::parse::decode_rad50_word;

#[derive(Debug, Clone)]
pub struct RstsEntry {
    pub name: String,
    pub owner_uic: (u16, u16),
    pub blocks: u16,
    pub status: u16,
}

#[derive(Debug, Clone)]
pub struct RstsBlockInfo {
    pub entries: Vec<RstsEntry>,
}

pub fn parse_block(block: &TapeBlock) -> Option<RstsBlockInfo> {
    let data = block.raw.as_ref();
    if data.len() < 32 {
        return None;
    }

    let mut entries = Vec::new();
    for chunk in data.chunks(32) {
        if chunk.len() < 16 {
            break;
        }
        let status = u16::from_le_bytes([chunk[0], chunk[1]]);
        let name_word1 = u16::from_le_bytes([chunk[2], chunk[3]]);
        let name_word2 = u16::from_le_bytes([chunk[4], chunk[5]]);
        let owner_uic = (
            u16::from_le_bytes([chunk[6], chunk[7]]),
            u16::from_le_bytes([chunk[8], chunk[9]]),
        );
        let blocks = u16::from_le_bytes([chunk[10], chunk[11]]);

        if status == 0 && name_word1 == 0 && name_word2 == 0 {
            continue;
        }

        let name = format!(
            "{}{}",
            decode_rad50_word(name_word1),
            decode_rad50_word(name_word2)
        );

        entries.push(RstsEntry {
            name,
            owner_uic,
            blocks,
            status,
        });
    }

    Some(RstsBlockInfo { entries })
}
