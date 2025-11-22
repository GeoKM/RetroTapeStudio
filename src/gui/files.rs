//! Files tab: displays assembled VMS files with metadata and a hex viewer modal for payloads.
use egui::{self, Align, Layout, ScrollArea, Vec2, Window};

use crate::backup::extract::VmsFile;
use crate::backup::vms::{format_protection, RecordFormat};
use crate::utils::hex::format_hex;
use crate::utils::text::{is_mostly_text, sanitize_display};

use super::state::AppState;

pub fn files_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Files");
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, file) in state.vms_files.iter().enumerate() {
                let sanitized_name = sanitize_display(&format!("{};{}", file.headers.full_name(), file.headers.version));
                let payload_len = total_size(file);
                let content_label = if is_mostly_text(&collect_payload_prefix(file, 512)) {
                    "(text)"
                } else {
                    "(binary)"
                };
                let file_type = file_type_label(file);
                ui.horizontal_wrapped(|ui| {
                    ui.label(sanitized_name);
                    ui.label(format!("blocks: {}", file.blocks.len()));
                    ui.label(format!("payload: {} bytes", payload_len));
                    ui.label(record_format_text(&file.headers.record_format));
                    ui.label(format!("type: {}", file_type));
                    ui.label(format!("UIC {:X}", file.headers.owner_uic));
                    ui.label(content_label);
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
                    let display_name =
                        sanitize_display(&format!("{};{}", file.headers.full_name(), file.headers.version));
                    ui.heading(display_name);
                    ui.label(format!("Path: {}", sanitize_display(&file.path)));
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
                    ui.label(format!("UIC: {:X}", file.headers.owner_uic));
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
            let name =
                sanitize_display(&format!("{};{}", file.headers.full_name(), file.headers.version));
            let mut close = false;
            let ctx = ui.ctx().clone();
            let max_height = ctx.available_rect().height() * 0.9;
            let mut open = true;
            Window::new("Hex Viewer")
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(ctx.available_rect().width() * 0.9, max_height))
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    });
                    ui.separator();
                    ui.heading(&name);
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.monospace(format_hex(&concatenated));
                        });
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            if close || !open {
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
