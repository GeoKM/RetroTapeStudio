use retro_tape_studio_v6_safe::backup::extract::assemble_files;
use retro_tape_studio_v6_safe::gui::input::set_tap_entries;
use retro_tape_studio_v6_safe::gui::state::AppState;
use retro_tape_studio_v6_safe::tap::legacy::{read_tap_records, TapDataKind};
use retro_tape_studio_v6_safe::tap::DetectedFormat;
use std::fs;

mod common;
use common::load_bb_h155c;

#[test]
fn gui_path_loads_real_vms_tape() {
    let bytes = load_bb_h155c();
    assert!(!bytes.is_empty(), "fixture should not be empty");

    let entries = read_tap_records(&bytes).expect("GUI parsing path should succeed");
    assert!(
        !entries.is_empty(),
        "expected parsed entries from BB-H155C-SE.tap"
    );
    let vms_blocks = entries
        .iter()
        .filter(|entry| matches!(entry.kind, TapDataKind::VmsBlock(_)))
        .count();
    assert!(vms_blocks > 0, "expected VMS blocks in parsed entries");

    let detected_format = entries
        .iter()
        .find(|entry| entry.detected_format == DetectedFormat::VmsBackup)
        .map(|entry| entry.detected_format)
        .expect("expected VMS BACKUP detection");
    assert_eq!(
        detected_format,
        DetectedFormat::VmsBackup,
        "expected detected format to stay VMS BACKUP"
    );

    let mut state = AppState::default();
    set_tap_entries(entries, &mut state);

    let summary = state
        .summary
        .as_ref()
        .expect("summary populated after loading TAP");
    assert!(
        summary.total_blocks > 0,
        "summary should include VMS blocks"
    );
    assert!(
        state.vms_fs.is_some(),
        "VMS directory tree should be built for files tab"
    );
    assert!(
        !state.tap_state.entries.is_empty(),
        "contents tab should have entries"
    );
    assert!(
        state
            .tap_state
            .entries
            .iter()
            .any(|entry| entry.detected_format == DetectedFormat::VmsBackup),
        "state should preserve detected VMS entries"
    );
    assert!(
        !state.vms_files.is_empty(),
        "files tab should have assembled VMS files"
    );
    assert!(
        state
            .vms_fs
            .as_ref()
            .map(|fs| !fs.root.children.is_empty() || !fs.root.files.is_empty())
            .unwrap_or(false),
        "files tree should include directories or files"
    );

    let assembled = assemble_files(&state.tap_state.entries);
    assert!(
        !assembled.is_empty(),
        "extraction tab should see assembled file payloads"
    );

    let output = format!(
        "entries={} vms_blocks={} vms_files={} blocks={} assembled={}",
        state.tap_state.entries.len(),
        vms_blocks,
        state.vms_files.len(),
        summary.total_blocks,
        assembled.len()
    );
    let _ = fs::create_dir_all("tests/output");
    fs::write("tests/output/gui_smoke.txt", output).expect("should write gui smoke output");
}
