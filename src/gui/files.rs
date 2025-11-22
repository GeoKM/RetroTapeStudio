//! Files tab: displays assembled VMS files with metadata and a hex viewer modal for payloads.
use egui::{self, ScrollArea, Window};

use crate::backup::extract::VmsFile;
use crate::backup::vms::{format_protection, RecordFormat};
use crate::utils::hex::format_hex;

use super::state::AppState;

pub fn files_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Files");
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, file) in state.vms_files.iter().enumerate() {
                let payload_len = total_size(file);
                let content_hint = text_preview(file);
                let file_type = file_type_label(file);
                ui.horizontal(|ui| {
                    ui.label(&file.name);
                    ui.label(format!("blocks: {}", file.blocks.len()));
                    ui.label(format!("payload: {} bytes", payload_len));
                    ui.label(record_format_text(&file.headers.record_format));
                    ui.label(format!("type: {}", file_type));
                    ui.label(content_hint);
                    if ui.button("View details").clicked() {
                        state.selected_file = Some(idx);
                        state.file_hex_viewer = None;
                    }
                });
                ui.separator();
            }
        });

    if let Some(idx) = state.selected_file {
        if let Some(file) = state.vms_files.get(idx) {
            let mut open_hex = false;
            let mut close_details = false;
            Window::new("File Details")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(&file.name);
                    ui.label(format!("Path: {}", file.path));
                    ui.label(format!(
                        "Record format: {}",
                        record_format_text(&file.headers.record_format)
                    ));
                    ui.label(format!(
                        "Protection: {}",
                        format_protection(file.headers.protection_mask)
                    ));
                    ui.label(format!("Owner UIC: {:X}", file.headers.owner_uic));
                    if let Some(ext) = &file.headers.extended {
                        ui.label(format!("Backup flags: {:?}", ext.backup_flags));
                        ui.label(format!(
                            "High precision time: {:?}",
                            ext.high_precision_timestamp
                        ));
                        ui.label(format!("ACP attrs: {:?}", ext.acp_attributes));
                        ui.label(format!("Journaling: {:?}", ext.journaling_flags));
                        ui.label(format!("File ID: {:?}", ext.file_id));
                        ui.label(format!("Sequence: {:?}", ext.sequence_number));
                    }
                    ui.separator();
                    ui.label("RMS Attributes");
                    ui.label(format!(
                        "RATTR: {} RATTNR: {} RATTNL: {}",
                        file.headers.rms.rattr, file.headers.rms.rattnr, file.headers.rms.rattnl
                    ));
                    ui.label(format!("Preview: {}", text_preview(file)));
                    ui.separator();
                    if ui.button("Open hex viewer").clicked() {
                        open_hex = true;
                    }
                    if ui.button("Close").clicked() {
                        close_details = true;
                    }
                });
            if close_details {
                state.selected_file = None;
                state.file_hex_viewer = None;
            } else if open_hex {
                state.file_hex_viewer = Some(idx);
            }
        } else {
            state.selected_file = None;
            state.file_hex_viewer = None;
        }
    }

    if let Some(idx) = state.file_hex_viewer {
        if let Some(file) = state.vms_files.get(idx) {
            let concatenated = collect_payload(file);
            let name = file.name.clone();
            let mut close = false;
            Window::new("Hex Viewer")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(&name);
                    ui.monospace(format_hex(&concatenated));
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            if close {
                state.file_hex_viewer = None;
            }
        } else {
            state.file_hex_viewer = None;
        }
    }
}

fn total_size(file: &VmsFile) -> usize {
    file.blocks.iter().map(|b| b.payload.len()).sum()
}

fn record_format_text(rfm: &RecordFormat) -> &'static str {
    match rfm {
        RecordFormat::Udf => "UDF",
        RecordFormat::Vfc => "VFC",
        RecordFormat::Var => "VAR",
        RecordFormat::Fix => "FIX",
        RecordFormat::Unknown(_) => "UNKNOWN",
    }
}

fn file_type_label(file: &VmsFile) -> String {
    if file.headers.file_type.is_empty() {
        "(unknown)".to_string()
    } else {
        file.headers.file_type.clone()
    }
}

fn text_preview(file: &VmsFile) -> String {
    let sample = collect_payload_prefix(file, 256);
    std::str::from_utf8(&sample)
        .ok()
        .and_then(|text| {
            let printable = text
                .chars()
                .all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t');
            if !printable {
                return None;
            }
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return None;
            }
            let snippet: String = trimmed.chars().take(80).collect();
            Some(if trimmed.len() > snippet.len() {
                format!("{snippet}...")
            } else {
                snippet
            })
        })
        .unwrap_or_else(|| "(binary)".to_string())
}

fn collect_payload(file: &VmsFile) -> Vec<u8> {
    file.blocks.iter().flat_map(|b| b.payload.clone()).collect()
}

fn collect_payload_prefix(file: &VmsFile, limit: usize) -> Vec<u8> {
    let mut data = Vec::new();
    for block in &file.blocks {
        if data.len() >= limit {
            break;
        }
        for byte in &block.payload {
            if data.len() >= limit {
                break;
            }
            data.push(*byte);
        }
    }
    data
}
