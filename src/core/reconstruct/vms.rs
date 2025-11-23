//! Convert raw FH2 block(s) into TapeFile metadata.

use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile, TapePath, VmsFileMetadata};
use crate::core::parse::vms::parse_vms_fh2;

/// Reconstruct all VMS files from classified blocks.
pub fn reconstruct_vms(blocks: &[TapeBlock]) -> Vec<TapeFile> {
    crate::core::vms::reconstruct::reconstruct_vms(blocks)
}

/// Build a TapeFile from a VMS FH2 header block and associated VBN blocks.
pub fn reconstruct_vms_file(
    fh2_block: &TapeBlock,
    vbn_blocks: &[TapeBlock],
    path: String,
) -> TapeFile {
    let meta = parse_vms_fh2(fh2_block).unwrap_or(default_vms_metadata());

    let indices: Vec<u32> = vbn_blocks.iter().map(|b| b.index).collect();
    let size_bytes = vbn_blocks.iter().map(|b| b.raw.len() as u64).sum::<u64>();

    TapeFile {
        format: crate::core::block::TapeFormat::Vms,
        path: TapePath::new(vec![path]),
        size_bytes,
        blocks: indices,
        metadata: FileMetadata::Vms(meta),
        children: Vec::new(),
    }
}

fn default_vms_metadata() -> VmsFileMetadata {
    VmsFileMetadata {
        file_id: (0, 0, 0),
        rev: 0,
        seq: 0,
        owner_uic: (0, 0),
        protection: 0,
        record_format: 0,
        record_attributes: 0,
        record_length: 0,
        file_type: "Unknown".to_string(),
        backup_flags: 0,
        creation_time: None,
        revision_time: None,
        expiration_time: None,
    }
}
