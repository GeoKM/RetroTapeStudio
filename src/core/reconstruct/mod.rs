pub mod builder;
pub mod vms;

use crate::core::block::{BlockClassification, TapeBlock, TapeFormat};
use crate::core::file::TapeFile;

pub fn reconstruct_all(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let mut out = Vec::new();
    let mut added_vms = false;

    match detect_dominant_format(blocks) {
        TapeFormat::Vms => {
            out.extend(vms::reconstruct_vms(blocks));
            added_vms = true;
        }
        TapeFormat::Rsx => out.extend(builder::reconstruct_rsx(blocks)),
        TapeFormat::Rt11 => out.extend(builder::reconstruct_rt11(blocks)),
        TapeFormat::Rsts => out.extend(builder::reconstruct_rsts(blocks)),
        _ => {}
    };

    if !added_vms
        && blocks
            .iter()
            .any(|b| matches!(b.classification, BlockClassification::Vms(_)))
    {
        out.extend(vms::reconstruct_vms(blocks));
    }

    out
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
        Some(3) => Vms,
        _ => Raw,
    }
}
