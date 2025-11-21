//! Shared GUI state structures and active tab tracking.
use crate::backup::extract::{VmsFile, VmsFileSystem};
use crate::gui::extraction::ExtractionState;
use crate::log::parse::LogData;
use crate::summary::SaveSetSummary;
use crate::tap::reader::TapEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    Contents,
    Extraction,
    Files,
    Summary,
    Log,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub tap_state: TapState,
    pub log_state: LogState,
    pub vms_files: Vec<VmsFile>,
    pub vms_fs: Option<VmsFileSystem>,
    pub selected_file: Option<usize>,
    pub summary: Option<SaveSetSummary>,
    pub summary_status: String,
    pub current_tab: MainTab,
    pub extraction: ExtractionState,
}

#[derive(Debug, Clone, Default)]
pub struct TapState {
    pub entries: Vec<TapEntry>,
    pub selected_entry: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct LogState {
    pub data: Option<LogData>,
    pub correlated: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tap_state: TapState::default(),
            log_state: LogState::default(),
            vms_files: Vec::new(),
            vms_fs: None,
            selected_file: None,
            summary: None,
            summary_status: String::new(),
            current_tab: MainTab::Contents,
            extraction: ExtractionState::default(),
        }
    }
}
