use std::fs;
use std::io;
use std::path::Path;

use crate::core::block::TapeBlock;
use crate::core::file::TapeFile;

use super::sanitize_filename;

pub fn extract_raw_file(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
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
    fs::write(path, &data)
}
