use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile};

use super::sanitize_filename;

pub fn extract(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    let path = outdir.join(sanitize_filename(&file.path.to_string_path()));

    if let FileMetadata::Rsx(meta) = &file.metadata {
        if meta.is_directory {
            fs::create_dir_all(&path)?;
            return Ok(());
        }
    }

    let mut data = Vec::new();
    for idx in &file.blocks {
        if let Some(b) = blocks.iter().find(|blk| blk.index == *idx) {
            data.extend_from_slice(b.raw.as_ref());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    f.write_all(&data)?;
    Ok(())
}
