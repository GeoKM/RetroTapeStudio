use retro_tape_studio_v6_safe::backup::extract::assemble_files;
use retro_tape_studio_v6_safe::gui::state::AppState;
use retro_tape_studio_v6_safe::log::parse::{LogData, LogEntry, LogLevel};
use retro_tape_studio_v6_safe::summary::compute_saveset_summary;
use retro_tape_studio_v6_safe::tap::legacy::{TapDataKind, TapEntry};
use retro_tape_studio_v6_safe::tap::DetectedFormat;
mod common;
use common::write_output;

#[test]
fn computes_counts_and_histograms() {
    let entries = vec![
        TapEntry {
            length: 12,
            kind: TapDataKind::Raw(vec![1, 2, 3]),
            log_level: None,
            detected_format: DetectedFormat::Raw,
        },
        TapEntry {
            length: 12,
            kind: TapDataKind::Raw(vec![4, 5, 6]),
            log_level: Some(LogLevel::Warning),
            detected_format: DetectedFormat::Raw,
        },
    ];
    let mut state = AppState::default();
    state.tap_state.entries = entries;
    state.log_state.data = Some(LogData {
        entries: vec![LogEntry {
            line: "Warning".into(),
            level: LogLevel::Warning,
        }],
        metadata: Default::default(),
    });

    let summary = compute_saveset_summary(&state);
    assert_eq!(
        summary.total_files,
        assemble_files(&state.tap_state.entries).len()
    );
    assert_eq!(summary.log_warnings, 1);
    write_output("summary", "summary.txt", &format!("{summary:?}"));
}
