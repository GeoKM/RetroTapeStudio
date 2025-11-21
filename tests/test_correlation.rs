use retro_tape_studio_v6_safe::log::parse::{correlate_log, LogData, LogEntry, LogLevel};
use retro_tape_studio_v6_safe::tap::reader::{TapDataKind, TapEntry};
use retro_tape_studio_v6_safe::tap::DetectedFormat;

#[test]
fn correlates_warning_to_entries() {
    let mut entries = vec![
        TapEntry {
            length: 10,
            kind: TapDataKind::Raw(vec![1, 2]),
            log_level: None,
            detected_format: DetectedFormat::Raw,
        },
        TapEntry {
            length: 10,
            kind: TapDataKind::Raw(vec![3, 4]),
            log_level: None,
            detected_format: DetectedFormat::Raw,
        },
    ];

    let log = LogData {
        entries: vec![LogEntry {
            line: "record 2 warning".into(),
            level: LogLevel::Warning,
        }],
        metadata: Default::default(),
    };

    correlate_log(&mut entries, &log);
    assert!(entries[1].log_level.is_some());
}
