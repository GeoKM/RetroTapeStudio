use crate::log::parse::LogData;
use crate::tap::reader::TapEntry;

#[derive(Debug, Clone)]
pub struct AppState {
    pub log: Option<LogData>,
    pub tap_entries: Vec<TapEntry>,
    pub selected_entry: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            log: None,
            tap_entries: Vec::new(),
            selected_entry: None,
        }
    }
}
