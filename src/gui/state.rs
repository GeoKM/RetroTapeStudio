use crate::log::parse::LogData;
use crate::tap::reader::TapEntry;
use crate::backup::extract::{VmsFile, VmsFileSystem};

#[derive(Debug, Clone)]
pub struct AppState {
    pub log: Option<LogData>,
    pub tap_entries: Vec<TapEntry>,
    pub selected_entry: Option<usize>,
    pub vms_files: Vec<VmsFile>,
    pub vms_fs: Option<VmsFileSystem>,
    pub selected_file: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            log: None,
            tap_entries: Vec::new(),
            selected_entry: None,
            vms_files: Vec::new(),
            vms_fs: None,
            selected_file: None,
        }
    }
}
