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

    let (is_dir, expected_len, prefix_path) = match &file.metadata {
        FileMetadata::Rsts(meta) => {
            let is_dir = meta.blocks == 0 || (meta.status & 0x8000) != 0;
            let expected_len = meta.blocks as usize * 512;
            let uic = format!("[{:o},{:o}]", meta.owner_uic.0, meta.owner_uic.1);
            (is_dir, expected_len, Some(uic))
        }
        _ => (false, 0, None),
    };

    if expected_len > 0 && data.len() > expected_len {
        data.truncate(expected_len);
    }

    let mut name = file.path.to_string_path();
    if let Some(uic) = prefix_path {
        if !name.starts_with(&uic) {
            name = format!("{}/{}", uic, name);
        }
    }

    let path = outdir.join(sanitize_filename(&name));

    if is_dir {
        fs::create_dir_all(&path)?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    f.write_all(&data)?;
    Ok(())
}
