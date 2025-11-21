//! LOG parsing: classify lines by severity, collect drive metadata, and correlate with TAP records.
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::tap::reader::TapEntry;
use crate::{TapeError, TapeResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub line: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogData {
    pub entries: Vec<LogEntry>,
    pub metadata: HashMap<String, String>,
}

/// Parse a TAP companion `.LOG` file, classifying lines by severity and
/// extracting drive metadata hints when present.
pub fn parse_log(path: &Path) -> TapeResult<LogData> {
    let content = fs::read_to_string(path)
        .map_err(|err| TapeError::Io(err))?;

    let mut entries = Vec::new();
    let mut metadata = HashMap::new();

    for line in content.lines() {
        // Capture metadata if present.
        parse_drive_metadata(line, &mut metadata);

        let level = if contains_any(line, &["ERROR", "BAD"]) {
            LogLevel::Error
        } else if contains_any(line, &["WARNING", "SKIP"]) {
            LogLevel::Warning
        } else {
            LogLevel::Info
        };

        entries.push(LogEntry {
            line: line.to_string(),
            level,
        });
    }

    Ok(LogData { entries, metadata })
}

/// Correlate log lines to TAP entries by record or block number, annotating entries with log levels.
pub fn correlate_log(entries: &mut [TapEntry], log: &LogData) {
    for log_entry in &log.entries {
        let level = match log_entry.level {
            LogLevel::Error => Some(LogLevel::Error),
            LogLevel::Warning => Some(LogLevel::Warning),
            LogLevel::Info => None,
        };

        let Some(level) = level else { continue };

        let idx = find_record_index(&log_entry.line).and_then(|n| n.checked_sub(1));
        if let Some(i) = idx {
            if let Some(entry) = entries.get_mut(i) {
                // Preserve the highest severity when multiple log lines hit the same record.
                match entry.log_level {
                    Some(LogLevel::Error) => {}
                    _ => entry.log_level = Some(level.clone()),
                }
            }
        }
    }
}

fn find_record_index(line: &str) -> Option<usize> {
    let lower = line.to_ascii_lowercase();
    for key in ["record", "block", "skipped"] {
        if let Some(pos) = lower.find(key) {
            let remainder = lower[pos + key.len()..].trim_start();
            let digits: String = remainder
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                if let Ok(n) = digits.parse::<usize>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let upper = haystack.to_ascii_uppercase();
    needles.iter().any(|needle| upper.contains(needle))
}

fn parse_drive_metadata(line: &str, metadata: &mut HashMap<String, String>) {
    for key in ["Tracks", "Density", "Blocks read"] {
        let pattern = format!("{} =", key);
        if let Some(idx) = line.find(&pattern) {
            // Extract substring after "key =" and trim.
            let value = line[idx + pattern.len()..].trim();
            metadata.insert(key.to_string(), value.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{correlate_log, parse_drive_metadata, parse_log, LogData, LogEntry, LogLevel};
    use std::collections::HashMap;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use crate::tap::reader::{TapDataKind, TapEntry};
    use crate::tap::DetectedFormat;

    fn temp_log_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        path.push(format!("retro_tape_log_test_{}_{}_{}.log", name, pid, nanos));
        path
    }

    #[test]
    fn classifies_levels() -> Result<(), io::Error> {
        let path = temp_log_path("levels");
        let content =
            "All good\nWARNING: slow block\nERROR reading\nSKIP block\nBAD checksum\nUnknown";
        fs::write(&path, content)?;

        let data = parse_log(&path).expect("should parse");
        assert_eq!(data.entries.len(), 6);
        assert!(matches!(data.entries[0].level, LogLevel::Info));
        assert!(matches!(data.entries[1].level, LogLevel::Warning));
        assert!(matches!(data.entries[2].level, LogLevel::Error));
        assert!(matches!(data.entries[3].level, LogLevel::Warning));
        assert!(matches!(data.entries[4].level, LogLevel::Error));
        assert!(matches!(data.entries[5].level, LogLevel::Info));
        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn captures_metadata() {
        let mut metadata = HashMap::new();
        parse_drive_metadata("Tracks = 80", &mut metadata);
        parse_drive_metadata("Density = DD", &mut metadata);
        parse_drive_metadata("Blocks read = 1024", &mut metadata);

        assert_eq!(metadata.get("Tracks").unwrap(), "80");
        assert_eq!(metadata.get("Density").unwrap(), "DD");
        assert_eq!(metadata.get("Blocks read").unwrap(), "1024");
    }

    #[test]
    fn parses_metadata_from_log() -> Result<(), io::Error> {
        let path = temp_log_path("metadata");
        fs::write(
            &path,
            "Tracks = 40\nDensity = SD\nBlocks read = 512\nAll good",
        )?;

        let data = parse_log(&path).expect("should parse");
        assert_eq!(data.metadata.get("Tracks").unwrap(), "40");
        assert_eq!(data.metadata.get("Density").unwrap(), "SD");
        assert_eq!(data.metadata.get("Blocks read").unwrap(), "512");
        assert_eq!(data.entries.len(), 4);
        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn correlates_record_numbers() {
        let mut entries = vec![
            TapEntry { length: 1, kind: TapDataKind::Raw(vec![]), log_level: None, detected_format: DetectedFormat::Raw },
            TapEntry { length: 1, kind: TapDataKind::Raw(vec![]), log_level: None, detected_format: DetectedFormat::Raw },
        ];

        let log = LogData {
            entries: vec![
                LogEntry { line: "Record 1 processed".into(), level: LogLevel::Info },
                LogEntry { line: "Record 2 ERROR".into(), level: LogLevel::Error },
            ],
            metadata: HashMap::new(),
        };

        correlate_log(&mut entries, &log);
        assert!(entries[0].log_level.is_none());
        assert!(matches!(entries[1].log_level, Some(LogLevel::Error)));
    }
}
