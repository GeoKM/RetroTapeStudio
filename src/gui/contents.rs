//! Contents tab: lists TAP entries with format and log badges plus a hex viewer popup.
use egui::{self, Align, Color32, Layout, ScrollArea, Vec2, Window};

use crate::log::parse::LogLevel;
use crate::tap::reader::{TapDataKind, TapEntry};
use crate::utils::hex::format_hex;
use crate::utils::text::sanitize_display;

use super::state::AppState;

/// Render a table of TAP entries inside the Contents tab.
pub fn contents_table(ui: &mut egui::Ui, entries: &[TapEntry], app_state: &mut AppState) {
    // Header row
    ui.horizontal(|ui| {
        ui.label("Index");
        ui.label("Length");
        ui.label("Format");
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
                    ui.colored_label(
                        format_color(entry.detected_format),
                        format!("{:?}", entry.detected_format),
                    );

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
                        app_state.tap_state.selected_entry = Some(idx);
                    }
                });
                ui.separator();
            }
        });

    if let Some(idx) = app_state.tap_state.selected_entry {
        if let Some(entry) = entries.get(idx) {
            let mut close = false;
            let mut open = true;
            let ctx = ui.ctx().clone();
            let max_height = ctx.available_rect().height() * 0.9;
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
                    ui.label(sanitize_display(&format!("Entry {}", idx)));
                    let bytes: Vec<u8> = match &entry.kind {
                        TapDataKind::Raw(data) => data.clone(),
                        TapDataKind::VmsBlock(block) => block.payload.clone(),
                    };
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.monospace(format_hex(&bytes));
                        });
                });
            if close || !open {
                app_state.tap_state.selected_entry = None;
            }
        } else {
            app_state.tap_state.selected_entry = None;
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

fn format_color(format: crate::tap::DetectedFormat) -> Color32 {
    match format {
        crate::tap::DetectedFormat::VmsBackup => Color32::GREEN,
        crate::tap::DetectedFormat::Rsx11m => Color32::YELLOW,
        crate::tap::DetectedFormat::Rt11 => Color32::from_rgb(0, 200, 200),
        crate::tap::DetectedFormat::RstsE => Color32::BLUE,
        crate::tap::DetectedFormat::Raw => Color32::GRAY,
    }
}
