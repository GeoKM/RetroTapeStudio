pub mod rsts;
pub mod rsx;
pub mod rt11;

use std::io;
use std::path::Path;

use crate::core::block::{TapeBlock, TapeFormat};
use crate::core::file::TapeFile;

pub fn extract_file(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    match file.format {
        TapeFormat::Rsx => rsx::extract(file, blocks, outdir),

        TapeFormat::Rt11 => rt11::extract(file, blocks, outdir),

        TapeFormat::Rsts => rsts::extract(file, blocks, outdir),

        TapeFormat::Vms => extract_vms(file, blocks, outdir),

        _ => extract_raw(file, blocks, outdir),
    }
}

fn extract_raw(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    let mut data = Vec::new();
    for idx in &file.blocks {
        if let Some(b) = blocks.iter().find(|blk| blk.index == *idx) {
            data.extend_from_slice(b.raw.as_ref());
        }
    }

    let name = sanitize_filename(&file.path.to_string_path());
    let path = outdir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    f.write_all(&data)?;
    Ok(())
}

fn sanitize_filename(s: &str) -> String {
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

fn extract_vms(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    // Wrapper around the existing VMS implementation.
    // Converts TapeFile -> fake minimal VmsFile for compatibility.
    // Later we will upgrade this in Stage 8.
    // For now this is shallow and safe.
    let mut payload = Vec::new();
    for idx in &file.blocks {
        if let Some(b) = blocks.iter().find(|blk| blk.index == *idx) {
            payload.extend_from_slice(b.raw.as_ref());
        }
    }

    // Just write raw for now -- Stage 8 will fully integrate VMS metadata.
    let filename = sanitize_filename(&file.path.to_string_path());
    let path = outdir.join(filename);
    std::fs::write(path, &payload)?;
    Ok(())
}
