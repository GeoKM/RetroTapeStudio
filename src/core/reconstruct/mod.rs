pub mod builder;

use crate::core::block::{BlockClassification, TapeBlock, TapeFormat};
use crate::core::file::TapeFile;

pub fn reconstruct_all(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    match detect_dominant_format(blocks) {
        TapeFormat::Rsx => builder::reconstruct_rsx(blocks),
        TapeFormat::Rt11 => builder::reconstruct_rt11(blocks),
        TapeFormat::Rsts => builder::reconstruct_rsts(blocks),
        _ => Vec::new(),
    }
}

fn detect_dominant_format(blocks: &[TapeBlock]) -> TapeFormat {
    use TapeFormat::*;
    let mut counts = [0usize; 4];
    for b in blocks {
        match b.classification {
            BlockClassification::Rsx(_) => counts[0] += 1,
            BlockClassification::Rt11(_) => counts[1] += 1,
            BlockClassification::Rsts(_) => counts[2] += 1,
            BlockClassification::Vms(_) => counts[3] += 1, // (unused now)
            _ => {}
        }
    }
    let max = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, v)| *v)
        .map(|(i, _)| i);
    match max {
        Some(0) => Rsx,
        Some(1) => Rt11,
        Some(2) => Rsts,
        _ => Raw,
    }
}
