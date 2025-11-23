use super::*;
use crate::core::block::TapeBlock;
use crate::core::file::TapeFile;
use std::fs;
use std::path::Path;

pub fn extract_vms_file(
    file: &TapeFile,
    blocks: &[TapeBlock],
    outdir: &Path,
) -> std::io::Result<()> {
    let mut data = Vec::new();

    for idx in &file.blocks {
        if let Some(b) = blocks.iter().find(|x| x.index == *idx) {
            data.extend_from_slice(&b.raw);
        }
    }

    let outfile = outdir.join(file.path.to_string_path());
    if let Some(parent) = outfile.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(outfile, &data)?;
    Ok(())
}
