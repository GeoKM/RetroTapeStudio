//! Log tab: renders parsed log metadata and color-coded entries.
use egui::{self, Color32, Grid, ScrollArea};

use crate::log::parse::{LogData, LogLevel};

/// Render parsed log data with metadata and per-line severity coloring.
pub fn draw_log(ui: &mut egui::Ui, log: &Option<LogData>) {
    match log {
        None => {
            ui.label("No log loaded");
        }
        Some(data) => {
            ui.heading("Metadata");
            if data.metadata.is_empty() {
                ui.label("No metadata found");
            } else {
                Grid::new("log_metadata_grid").num_columns(2).show(ui, |ui| {
                    for (key, value) in &data.metadata {
                        ui.label(key);
                        ui.label(value);
                        ui.end_row();
                    }
                });
            }

            ui.separator();
            ui.heading("Entries");

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &data.entries {
                        let text = &entry.line;
                        match entry.level {
                            LogLevel::Info => {
                                ui.label(text);
                            }
                            LogLevel::Warning => {
                                ui.colored_label(Color32::YELLOW, text);
                            }
                            LogLevel::Error => {
                                ui.colored_label(Color32::RED, text);
                            }
                        }
                    }
                });
        }
    }
}
