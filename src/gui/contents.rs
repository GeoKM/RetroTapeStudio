use egui::{self, Color32, ScrollArea, Window};

use crate::log::parse::LogLevel;
use crate::tap::reader::{TapDataKind, TapEntry};
use crate::utils::hex::format_hex;

use super::state::AppState;

/// Render a table of TAP entries inside the Contents tab.
pub fn contents_table(ui: &mut egui::Ui, entries: &[TapEntry], app_state: &mut AppState) {
    // Header row
    ui.horizontal(|ui| {
        ui.label("Index");
        ui.label("Length");
        ui.label("Kind");
        ui.label("Phase");
        ui.label("Sequence");
        ui.label("Checksum");
        ui.label("Log");
        ui.label("View");
    });
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, entry) in entries.iter().enumerate() {
                ui.horizontal(|ui| {
                    if let Some(color) = badge_color(entry.log_level.as_ref()) {
                        ui.colored_label(color, "!");
                    } else {
                        ui.label(" ");
                    }
                    ui.label(idx.to_string());
                    ui.label(entry.length.to_string());

                    match &entry.kind {
                        TapDataKind::Raw(data) => {
                            ui.label("Raw");
                            ui.label("-");
                            ui.label("-");
                            ui.label(format!("{} bytes", data.len()));
                        }
                        TapDataKind::VmsBlock(block) => {
                            ui.label("VMS");
                            ui.label(block.phase.to_string());
                            ui.label(block.sequence_number.to_string());
                            ui.label(format!("{:#06X}", block.checksum));
                        }
                    }

                    if ui.button("View").clicked() {
                        app_state.selected_entry = Some(idx);
                    }
                });
                ui.separator();
            }
        });

    if let Some(idx) = app_state.selected_entry {
        if let Some(entry) = entries.get(idx) {
            Window::new("Hex Viewer")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    let bytes: Vec<u8> = match &entry.kind {
                        TapDataKind::Raw(data) => data.clone(),
                        TapDataKind::VmsBlock(block) => block.payload.clone(),
                    };
                    ui.monospace(format_hex(&bytes));
                    if ui.button("Close").clicked() {
                        app_state.selected_entry = None;
                    }
                });
        } else {
            app_state.selected_entry = None;
        }
    }
}

fn badge_color(level: Option<&LogLevel>) -> Option<Color32> {
    match level {
        Some(LogLevel::Error) => Some(Color32::RED),
        Some(LogLevel::Warning) => Some(Color32::YELLOW),
        _ => None,
    }
}
