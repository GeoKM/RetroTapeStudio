use std::fs;
use std::path::PathBuf;

use common::write_output;
use retro_tape_studio_v6_safe::log::parse::{parse_log, LogLevel};
mod common;

#[test]
fn parses_log_levels_and_metadata() {
    let tmp = temp_log(
        "Tracks = 40\nDensity = DD\nBlocks read = 100\nWARNING slow\nERROR fail\ninfo line",
    );
    let data = parse_log(&tmp).expect("parse log");
    assert_eq!(data.entries.len(), 6);
    assert_eq!(data.log_warnings(), 1);
    assert_eq!(data.log_errors(), 1);
    assert_eq!(data.metadata.get("Tracks").unwrap(), "40");
    assert_eq!(data.metadata.get("Density").unwrap(), "DD");
    assert_eq!(data.metadata.get("Blocks read").unwrap(), "100");
    let _ = fs::remove_file(tmp);
    write_output(
        "log",
        "log_metadata.txt",
        &format!(
            "warnings={} errors={} tracks={:?} density={:?}",
            data.log_warnings(),
            data.log_errors(),
            data.metadata.get("Tracks"),
            data.metadata.get("Density")
        ),
    );
}

trait LogCounts {
    fn log_warnings(&self) -> usize;
    fn log_errors(&self) -> usize;
}

impl LogCounts for retro_tape_studio_v6_safe::log::parse::LogData {
    fn log_warnings(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Warning))
            .count()
    }
    fn log_errors(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .count()
    }
}

fn temp_log(content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "retro_tape_log_{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, content).unwrap();
    path
}
