use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::core::block::TapeBlock;
use crate::core::file::{FileMetadata, TapeFile};

use super::sanitize_filename;

pub fn extract(file: &TapeFile, blocks: &[TapeBlock], outdir: &Path) -> io::Result<()> {
    let mut data = Vec::new();
    for idx in &file.blocks {
        if let Some(b) = blocks.iter().find(|blk| blk.index == *idx) {
            data.extend_from_slice(b.raw.as_ref());
        }
    }

    let filename = match &file.metadata {
        FileMetadata::Rt11(meta) => {
            let expected_len = meta.length_blocks as usize * 512;
            if expected_len < data.len() {
                data.truncate(expected_len);
            }
            if meta.ext.is_empty() {
                file.path.to_string_path()
            } else {
                format!("{}.{}", file.path.to_string_path(), meta.ext)
            }
        }
        _ => file.path.to_string_path(),
    };

    let path = outdir.join(sanitize_filename(&filename));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    f.write_all(&data)?;
    Ok(())
}
