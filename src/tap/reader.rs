use std::fs::File;
use std::io::{self, Read};
use std::sync::Arc;

use crate::core::block::{BlockClassification, TapeBlock};

pub fn read_tap_blocks(path: &str) -> io::Result<Vec<TapeBlock>> {
    let mut f = File::open(path)?;
    let mut data = Vec::new();
    f.read_to_end(&mut data)?;

    let mut offset = 0usize;
    let mut index = 0u32;
    let mut blocks = Vec::new();

    while offset + 4 <= data.len() {
        // Read record length (little endian)
        let len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;

        offset += 4;

        // Safety: If len is too large or block incomplete -> break cleanly
        if len == 0 || offset + len > data.len() {
            break;
        }

        let block = Arc::<[u8]>::from(&data[offset..offset + len]);

        blocks.push(TapeBlock {
            index,
            size: len,
            raw: block,
            classification: BlockClassification::Unknown,
        });

        offset += len;

        // Skip trailing length (TAP trailer)
        if offset + 4 <= data.len() {
            offset += 4;
        } else {
            break;
        }

        index += 1;
    }

    Ok(blocks)
}
