use std::path::Path;

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
    if let Some(log) = &state.log {
        correlate_log(&mut state.tap_entries, log);
    }
}
