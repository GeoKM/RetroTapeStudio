//! Contents tab: lists reconstructed tape files with quick hex viewing.
use egui::{self, Align, Layout, ScrollArea, Vec2, Window};
use rfd::FileDialog;

use crate::core::extract::extract_file;
use crate::utils::hex::format_hex_with_ascii;
use crate::utils::text::sanitize_display;

use super::files::{collect_block_bytes, describe_metadata, flatten_files_tree};
use super::state::AppState;

/// Render a table of reconstructed files inside the Contents tab.
pub fn contents_table(ui: &mut egui::Ui, app_state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label("Idx");
        ui.add_space(6.0);
        ui.label("Type");
        ui.add_space(6.0);
        ui.label("Path");
        ui.add_space(6.0);
        ui.label("Format");
        ui.add_space(6.0);
        ui.label("Size");
        ui.add_space(6.0);
        ui.label("Blocks");
        ui.add_space(6.0);
        ui.label("Actions");
    });
    ui.separator();

    let flattened = flatten_files_tree(&app_state.files, 0);
    if flattened.is_empty() {
        ui.label("No reconstructed files available.");
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, (file, depth)) in flattened.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:>4}", idx));
                    ui.add_space(8.0);
                    ui.add_space(*depth as f32 * 10.0);
                    let icon = if file.children.is_empty() {
                        "\u{1F4C4}"
                    } else {
                        "\u{1F4C1}"
                    };
                    ui.label(icon);
                    ui.add_space(6.0);
                    ui.label(sanitize_display(&file.path.to_string_path()));
                    ui.add_space(8.0);
                    ui.label(format!("{:?}", file.format));
                    ui.add_space(8.0);
                    ui.label(format!("{} bytes", file.size_bytes));
                    ui.add_space(8.0);
                    ui.label(format!("{}", file.blocks.len()));
                    ui.add_space(8.0);
                    if ui.button("Extract").clicked() {
                        if let Some(dir) = FileDialog::new().pick_folder() {
                            match extract_file(file, &app_state.blocks, dir.as_path()) {
                                Ok(_) => {
                                    app_state.summary_status =
                                        format!("Extracted to {}", dir.display());
                                }
                                Err(err) => {
                                    app_state.summary_status = format!("Extract failed: {}", err);
                                }
                            }
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
        if let Some((file, _)) = flattened.get(idx) {
            let mut close = false;
            let ctx = ui.ctx().clone();
            let max_height = ctx.available_rect().height() * 0.9;
            let mut open = true;
            let bytes = collect_block_bytes(file, &app_state.blocks);
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
                    ui.label(sanitize_display(&file.path.to_string_path()));
                    ui.label(format!("Format: {:?}", file.format));
                    ui.label(format!("Size: {} bytes", file.size_bytes));
                    ui.label(format!("Blocks: {}", file.blocks.len()));
                    ui.separator();
                    ui.label("Metadata:");
                    for line in describe_metadata(file) {
                        ui.label(line);
                    }
                    ui.separator();
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.monospace(format_hex_with_ascii(&bytes));
                        });
                });
            if close || !open {
                app_state.tap_state.selected_entry = None;
            }
        } else {
            app_state.tap_state.selected_entry = None;
        }
    }

    if !app_state.summary_status.is_empty() {
        ui.separator();
        ui.label(&app_state.summary_status);
    }
}
