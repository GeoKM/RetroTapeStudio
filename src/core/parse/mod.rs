pub mod rsts;
pub mod rsx;
pub mod rt11;
pub mod vms;

use crate::core::block::{BlockClassification, TapeBlock};

#[derive(Debug, Clone)]
pub enum ParsedBlock {
    Rsx(self::rsx::RsxBlockInfo),
    Rt11(self::rt11::Rt11BlockInfo),
    Rsts(self::rsts::RstsBlockInfo),
}

pub fn parse_classified_block(block: &TapeBlock) -> Option<ParsedBlock> {
    match &block.classification {
        BlockClassification::Rsx(_) => rsx::parse_block(block).map(ParsedBlock::Rsx),
        BlockClassification::Rt11(_) => rt11::parse_block(block).map(ParsedBlock::Rt11),
        BlockClassification::Rsts(_) => rsts::parse_block(block).map(ParsedBlock::Rsts),
        _ => None,
    }
}

fn decode_rad50_word(word: u16) -> String {
    const RAD50_TABLE: &[u8] = b" ABCDEFGHIJKLMNOPQRSTUVWXYZ$.%0123456789";
    let a = (word / 1600) as usize;
    let b = ((word % 1600) / 40) as usize;
    let c = (word % 40) as usize;
    let mut out = String::new();
    for idx in [a, b, c] {
        if let Some(&ch) = RAD50_TABLE.get(idx) {
            if ch != b' ' {
                out.push(ch as char);
            }
        }
    }
    out
}
