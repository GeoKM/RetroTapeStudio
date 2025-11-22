//! Shared test utilities and fixture loading helpers.
//! External fixtures should be placed under `tests/data/`; outputs belong in `tests/output/`.
use std::fs;
use std::path::{Path, PathBuf};

use retro_tape_studio_v6_safe::log::parse::parse_log;
use retro_tape_studio_v6_safe::tap::reader::{read_tap_entry, TapEntry};
use retro_tape_studio_v6_safe::TapeResult;

pub fn fixture_path(name: &str) -> PathBuf {
    Path::new("tests").join("data").join(name)
}

pub fn load_tap_fixture(name: &str) -> Vec<u8> {
    fs::read(fixture_path(name)).expect("fixture TAP missing")
}

/// Load the BB-H155C-SE VMS BACKUP tape fixture.
pub fn load_bb_h155c() -> Vec<u8> {
    load_tap_fixture("BB-H155C-SE.tap")
}

pub fn load_log_fixture(name: &str) -> String {
    fs::read_to_string(fixture_path(name)).expect("fixture LOG missing")
}

/// Chunk a TAP file into entries using 512-byte blocks.
pub fn read_tap_file_with_chunks(bytes: &[u8]) -> TapeResult<Vec<TapEntry>> {
    read_tap_with_chunks(bytes, 512)
}

/// Chunk a TAP file into entries using a configurable block size.
pub fn read_tap_with_chunks(bytes: &[u8], chunk_size: usize) -> TapeResult<Vec<TapEntry>> {
    let mut entries = Vec::new();
    for chunk in bytes.chunks(chunk_size) {
        if chunk.is_empty() {
            continue;
        }
        entries.push(read_tap_entry(chunk)?);
    }
    Ok(entries)
}

pub fn parse_log_path(path: &Path) -> TapeResult<retro_tape_studio_v6_safe::log::parse::LogData> {
    parse_log(path)
}

/// Ensure an output subdirectory exists under tests/output and return its path.
pub fn ensure_output_dir(sub: &str) -> PathBuf {
    let dir = Path::new("tests").join("output").join(sub);
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Write a UTF-8 text file into tests/output/<sub>/<name> for diagnostics.
pub fn write_output(sub: &str, name: &str, contents: &str) {
    let dir = ensure_output_dir(sub);
    let path = dir.join(name);
    let _ = fs::write(path, contents);
}
