//! Input helpers: load TAP/LOG inputs, correlate data, and refresh shared state.
use std::path::Path;
use crate::backup::extract::{assemble_vms_files, build_directory_tree};
use crate::log::parse::{correlate_log, parse_log};
use crate::summary::compute_saveset_summary;
use crate::tap::reader::TapEntry;
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

/// Store TAP entries (e.g., after reading a TAP image) and correlate with any loaded log.
pub fn set_tap_entries(entries: Vec<TapEntry>, state: &mut AppState) {
    state.tap_state.entries = entries;
    state.tap_state.selected_entry = None;
    state.selected_file = None;
    // Build VMS file structures for Files tab.
    state.vms_files = assemble_vms_files(&state.tap_state.entries);
    state.vms_fs = Some(build_directory_tree(&state.vms_files));
    if let Some(log) = &state.log_state.data {
        correlate_log(&mut state.tap_state.entries, log);
        state.log_state.correlated = true;
    }
    state.summary = Some(compute_saveset_summary(state));
}
