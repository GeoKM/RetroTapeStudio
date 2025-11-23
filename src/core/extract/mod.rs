pub mod raw;
pub mod rsts;
pub mod rsx;
pub mod rt11;

use std::io;
use std::path::Path;

use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile};

/// Select the correct extraction backend based on metadata.
pub fn extract_file(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(outdir)?;
    match &file.metadata {
        FileMetadata::Rsx(_) => rsx::extract_rsx_file(file, blocks, outdir),
        FileMetadata::Rt11(_) => rt11::extract_rt11_file(file, blocks, outdir),
        FileMetadata::Rsts(_) => rsts::extract_rsts_file(file, blocks, outdir),
        FileMetadata::Vms(_) => crate::core::vms::extract::extract_vms_file(file, blocks, outdir),
        FileMetadata::Raw => raw::extract_raw_file(file, blocks, outdir),
    }
}

pub(crate) fn sanitize_filename(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' || ch == '/' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}
