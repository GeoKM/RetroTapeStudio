//! VMS file reconstruction layer.
//! Turns FH2/XH2/XH3/VBN collections into TapeFile nodes.

use std::collections::BTreeMap;

use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile, TapePath, VmsFileMetadata};
use crate::core::vms::block::Fh2Record;
use crate::core::vms::{collect_vms_blocks, VmsCollected};

/// Main entry point for VMS reconstruction.
/// Collects parsed VMS structures from blocks, then builds file entries.
pub fn reconstruct_vms(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    let collected = collect_vms_blocks(blocks);
    reconstruct_vms_from_collected(&collected)
}

/// Reconstruct files from a pre-collected VMS map.
pub fn reconstruct_vms_from_collected(collected: &BTreeMap<u32, VmsCollected>) -> Vec<TapeFile> {
    let mut out = Vec::new();

    for (_fh2_index, group) in collected.iter() {
        if let Some(fh2) = &group.fh2 {
            let file = build_file_from_group(fh2, group);
            out.push(file);
        }
    }

    out
}

/// Turn a VMS FH2 + optional XH2/XH3 + VBN list into a concrete TapeFile.
fn build_file_from_group(fh2: &Fh2Record, group: &VmsCollected) -> TapeFile {
    // File path construction:
    // VMS savesets do NOT store directory paths.
    // So the filename is flat: NAME.TYPE;VERSION
    let full = build_vms_fullname(fh2);

    // Collect block identifiers from VBNs, sorted by VBN.
    let mut indices: Vec<u32> = group.vbn.iter().map(|v| v.vbn).collect();
    indices.sort_unstable();

    // Estimate final size (approximate). VBN payload length is arbitrary, but normally 512-byte blocks.
    let size = group
        .vbn
        .iter()
        .map(|vb| vb.payload.len() as u64)
        .sum::<u64>();

    TapeFile {
        format: crate::core::block::TapeFormat::Vms,
        path: TapePath::new(vec![full]),
        size_bytes: size,
        blocks: indices,
        metadata: FileMetadata::Vms(VmsFileMetadata { placeholder: false }),
        children: Vec::new(),
    }
}

/// Build a canonical VMS filename: NAME.TYPE;VERSION
fn build_vms_fullname(fh2: &Fh2Record) -> String {
    let mut s = String::new();
    s.push_str(fh2.file_name.trim());

    if !fh2.file_type.trim().is_empty() {
        s.push('.');
        s.push_str(fh2.file_type.trim());
    }

    if fh2.version != 0 {
        s.push(';');
        s.push_str(&fh2.version.to_string());
    }

    s
}
