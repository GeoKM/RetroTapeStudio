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
                ui.horizontal(|ui| {
                    ui.label(&file.name);
                    ui.label(&file.path);
                    ui.label(format!("{} bytes", total_size(file)));
                    ui.label(record_format_text(&file.headers.record_format));
                    ui.label(file.headers.creation_date.to_string());
                    ui.label(file.headers.revision_date.to_string());
                    ui.label(format_protection(file.headers.protection_mask));
                    ui.label(format!("UIC {:X}", file.headers.owner_uic));
                    if ui.button("View details").clicked() {
                        state.selected_file = Some(idx);
                    }
                });
                ui.separator();
            }
        });

    if let Some(idx) = state.selected_file {
        if let Some(file) = state.vms_files.get(idx) {
            Window::new("File Details")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(&file.name);
                    ui.label(format!("Path: {}", file.path));
                    ui.label(format!("Record format: {}", record_format_text(&file.headers.record_format)));
                    ui.label(format!("Protection: {}", format_protection(file.headers.protection_mask)));
                    ui.label(format!("Owner UIC: {:X}", file.headers.owner_uic));
                    if let Some(ext) = &file.headers.extended {
                        ui.label(format!("Backup flags: {:?}", ext.backup_flags));
                        ui.label(format!("High precision time: {:?}", ext.high_precision_timestamp));
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
                    ui.separator();
                    ui.label("Blocks hex dump");
                    let concatenated: Vec<u8> = file
                        .blocks
                        .iter()
                        .flat_map(|b| b.payload.clone())
                        .collect();
                    ui.monospace(format_hex(&concatenated));
                    if ui.button("Close").clicked() {
                        state.selected_file = None;
                    }
                });
        } else {
            state.selected_file = None;
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
