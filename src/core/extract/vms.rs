//! VMS extraction layer.
//! Takes reconstructed TapeFile (VMS) + VBN blocks -> real output files.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::core::block::TapeBlock;
use crate::core::extract::sanitize_filename;
use crate::core::file::{FileMetadata, TapeFile};

/// Extract a VMS file by concatenating VBN payloads in sorted VBN order.
pub fn extract_vms_file(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    match &file.metadata {
        FileMetadata::Vms(_) => {}
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "extract_vms_file() called for non-VMS file",
            ))
        }
    }

    let name = sanitize_filename(&file.path.to_string_path());
    let path = outdir.join(name);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut block_indices = file.blocks.clone();
    block_indices.sort_unstable();

    let mut buffer = Vec::new();
    for idx in block_indices {
        if let Some(b) = blocks.iter().find(|blk| blk.index == idx) {
            buffer.extend_from_slice(b.raw.as_ref());
        } else {
            eprintln!("Warning: missing VBN block index {}", idx);
        }
    }

    let mut f = fs::File::create(&path)?;
    f.write_all(&buffer)?;
    Ok(())
}

/// Dispatcher helper for VMS (called from extract_file).
pub fn extract_vms_dispatch(
    file: &TapeFile,
    blocks: &[TapeBlock],
    outdir: &Path,
) -> io::Result<()> {
    extract_vms_file(file, blocks, outdir)
}
