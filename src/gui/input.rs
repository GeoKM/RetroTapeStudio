use std::path::Path;

use crate::backup::extract::{assemble_vms_files, build_directory_tree};
use crate::log::parse::{correlate_log, parse_log};
use crate::tap::reader::TapEntry;

use super::state::AppState;

/// Load a `.LOG` file selected in the Input tab and store parsed data.
pub fn load_log_file(path: &Path, state: &mut AppState) -> Result<(), String> {
    let data = parse_log(path)?;
    correlate_log(&mut state.tap_entries, &data);
    state.log = Some(data);
    Ok(())
}

/// Store TAP entries (e.g., after reading a TAP image) and correlate with any loaded log.
pub fn set_tap_entries(entries: Vec<TapEntry>, state: &mut AppState) {
    state.tap_entries = entries;
    // Build VMS file structures for Files tab.
    state.vms_files = assemble_vms_files(&state.tap_entries);
    state.vms_fs = Some(build_directory_tree(&state.vms_files));
    if let Some(log) = &state.log {
        correlate_log(&mut state.tap_entries, log);
    }
}
