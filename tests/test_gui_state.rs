use retro_tape_studio_v6_safe::gui::input::{load_log_file, set_tap_entries};
use retro_tape_studio_v6_safe::gui::state::AppState;
use retro_tape_studio_v6_safe::log::parse::LogLevel;
use retro_tape_studio_v6_safe::tap::reader::{TapDataKind, TapEntry};
use retro_tape_studio_v6_safe::tap::DetectedFormat;
use std::fs;
use std::path::PathBuf;

#[test]
fn loading_tap_and_log_resets_selection() {
    let mut state = AppState::default();
    state.tap_state.selected_entry = Some(0);
    state.selected_file = Some(1);

    let entries = vec![TapEntry {
        length: 4,
        kind: TapDataKind::Raw(vec![1, 2, 3, 4]),
        log_level: None,
        detected_format: DetectedFormat::Raw,
    }];
    set_tap_entries(entries, &mut state);
    assert!(state.tap_state.selected_entry.is_none());
    assert!(state.selected_file.is_none());
}

#[test]
fn loading_log_correlates() {
    let mut state = AppState::default();
    let entries = vec![TapEntry {
        length: 4,
        kind: TapDataKind::Raw(vec![1, 2, 3, 4]),
        log_level: None,
        detected_format: DetectedFormat::Raw,
    }];
    set_tap_entries(entries, &mut state);

    let path = temp_log("record 1 warning");
    load_log_file(&path, &mut state).unwrap();
    assert!(matches!(
        state.tap_state.entries[0].log_level,
        Some(LogLevel::Warning) | Some(LogLevel::Error)
    ));
    let _ = fs::remove_file(path);
}

fn temp_log(content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "retro_tape_gui_state_{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, content).unwrap();
    path
}
