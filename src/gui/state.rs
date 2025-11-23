//! Shared GUI state structures and active tab tracking.
use crate::backup::extract::{VmsFile, VmsFileSystem};
use crate::core::block::{TapeBlock, TapeFormat};
use crate::core::file::TapeFile;
use crate::gui::extraction::ExtractionState;
use crate::log::parse::LogData;
use crate::summary::SaveSetSummary;
use crate::tap::legacy::TapEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    Input,
    Contents,
    Extraction,
    Files,
    Summary,
    Log,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub blocks: Vec<TapeBlock>,
    pub detected_format: TapeFormat,
    pub files: Vec<TapeFile>,
    pub tap_state: TapState,
    pub log_state: LogState,
    pub vms_files: Vec<VmsFile>,
    pub vms_fs: Option<VmsFileSystem>,
    pub selected_file: Option<usize>,
    pub file_hex_viewer: Option<usize>,
    pub summary: Option<SaveSetSummary>,
    pub summary_status: String,
    pub current_tab: MainTab,
    pub extraction: ExtractionState,
}

#[derive(Debug, Clone)]
pub struct TapState {
    pub entries: Vec<TapEntry>,
    pub selected_entry: Option<usize>,
}

impl Default for TapState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selected_entry: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogState {
    pub data: Option<LogData>,
    pub correlated: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            detected_format: TapeFormat::Unknown,
            files: Vec::new(),
            tap_state: TapState::default(),
            log_state: LogState::default(),
            vms_files: Vec::new(),
            vms_fs: None,
            selected_file: None,
            file_hex_viewer: None,
            summary: None,
            summary_status: String::new(),
            current_tab: MainTab::Contents,
            extraction: ExtractionState::default(),
        }
    }
}
