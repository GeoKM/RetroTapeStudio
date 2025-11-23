//! Input helpers and tab UI: load TAP/LOG inputs, correlate data, and refresh shared state.
use std::fs;
use std::path::Path;

use egui;
use rfd::FileDialog;

use crate::backup::extract::{assemble_vms_files, build_directory_tree};
use crate::core::reconstruct::reconstruct_all;
use crate::log::parse::{correlate_log, parse_log};
use crate::summary::compute_saveset_summary;
use crate::tap::legacy::{read_tap_records, TapEntry};
use crate::tap::reader::read_tap_blocks;
use crate::TapeResult;

use super::state::AppState;

/// Load a `.LOG` file selected in the Input tab and store parsed data.
pub fn load_log_file(path: &Path, state: &mut AppState) -> TapeResult<()> {
    let data = parse_log(path)?;
    correlate_log(&mut state.tap_state.entries, &data);
    state.log_state.data = Some(data);
    state.log_state.correlated = true;
    state.tap_state.selected_entry = None;
    state.selected_file = None;
    state.summary = Some(compute_saveset_summary(state));
    Ok(())
}

/// Parse a TAP file into entries using simple 512-byte chunks.
pub fn parse_tap_file(path: &Path) -> TapeResult<Vec<TapEntry>> {
    let data = fs::read(path)?;
    if data.is_empty() {
        return Err(crate::TapeError::Parse("empty TAP file".into()));
    }
    read_tap_records(&data)
}

/// Store TAP entries (e.g., after reading a TAP image) and correlate with any loaded log.
pub fn set_tap_entries(entries: Vec<TapEntry>, state: &mut AppState) {
    state.tap_state.entries = entries;
    state.tap_state.selected_entry = None;
    state.selected_file = None;
    state.file_hex_viewer = None;
    state.files = reconstruct_all(&state.blocks);
    // Build VMS file structures for Files tab.
    state.vms_files = assemble_vms_files(&state.tap_state.entries);
    state.vms_fs = if state.vms_files.is_empty() {
        None
    } else {
        Some(build_directory_tree(&state.vms_files))
    };
    if let Some(log) = &state.log_state.data {
        correlate_log(&mut state.tap_state.entries, log);
        state.log_state.correlated = true;
    }
    state.summary = Some(compute_saveset_summary(state));
}

/// Render the Input tab with file pickers for TAP/LOG.
pub fn input_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Load media");
    ui.horizontal(|ui| {
        if ui.button("Load TAP file").clicked() {
            if let Some(path) = FileDialog::new().add_filter("TAP", &["tap"]).pick_file() {
                let mut tap_status = None;
                if let Some(path_str) = path.to_str() {
                    match read_tap_blocks(path_str) {
                        Ok(mut blocks) => {
                            let detected = crate::core::detect::analyze_blocks(&mut blocks);
                            state.blocks = blocks;
                            state.detected_format = detected;
                            state.files = reconstruct_all(&state.blocks);
                        }
                        Err(err) => tap_status = Some(format!("TAP load failed: {err}")),
                    }
                } else {
                    tap_status = Some("TAP load failed: invalid path encoding".to_string());
                }

                match parse_tap_file(&path) {
                    Ok(entries) => {
                        set_tap_entries(entries, state);
                        state.summary_status = format!("Loaded TAP {}", path.display());
                    }
                    Err(err) => tap_status = Some(format!("TAP load failed: {err}")),
                }

                if let Some(status) = tap_status {
                    state.summary_status = status;
                }
            }
        }
        if ui.button("Load LOG file").clicked() {
            if let Some(path) = FileDialog::new().add_filter("LOG", &["log"]).pick_file() {
                match load_log_file(&path, state) {
                    Ok(_) => state.summary_status = format!("Loaded LOG {}", path.display()),
                    Err(err) => state.summary_status = format!("LOG load failed: {err}"),
                }
            }
        }
    });

    ui.separator();
    ui.label(&state.summary_status);
}
