//! Files tab: displays reconstructed file tree with metadata and hex viewers.
use egui::{self, Align, Layout, ScrollArea, Vec2, Window};
use rfd::FileDialog;

use crate::core::block::TapeBlock;
use crate::core::extract::extract_file;
use crate::core::file::{FileMetadata, TapeFile};
use crate::utils::hex::format_hex;
use crate::utils::text::sanitize_display;

use super::state::AppState;

pub fn files_tab(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Files");
    ui.separator();

    let flattened = flatten_files_tree(&state.files, 0);

    if flattened.is_empty() {
        ui.label("No reconstructed files available.");
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, (file, depth)) in flattened.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_space(*depth as f32 * 10.0);
                    let label = if file.children.is_empty() {
                        "File"
                    } else {
                        "Dir"
                    };
                    ui.label(label);
                    ui.label(sanitize_display(&file.path.to_string_path()));
                    ui.label(format!("{:?}", file.format));
                    ui.label(format!("{} bytes", file.size_bytes));
                    ui.label(format!("blocks: {}", file.blocks.len()));
                    if ui.button("Extract").clicked() {
                        if let Some(dir) = FileDialog::new().pick_folder() {
                            match extract_file(file, &state.blocks, dir.as_path()) {
                                Ok(_) => {
                                    state.summary_status =
                                        format!("Extracted to {}", dir.display());
                                }
                                Err(err) => {
                                    state.summary_status = format!("Extract failed: {}", err);
                                }
                            }
                        }
                    }
                    if ui.button("Details").clicked() {
                        state.selected_file = Some(idx);
                        state.file_hex_viewer = None;
                    }
                    if !file.blocks.is_empty() && ui.button("Hex").clicked() {
                        state.file_hex_viewer = Some(idx);
                        state.selected_file = None;
                    }
                });
                ui.separator();
            }
        });

    if let Some(idx) = state.selected_file {
        if let Some((file, _)) = flattened.get(idx) {
            let mut open_hex = false;
            let mut close_details = false;
            Window::new("File Details")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(sanitize_display(&file.path.to_string_path()));
                    ui.label(format!("{:?}", file.format));
                    ui.label(format!("Size: {} bytes", file.size_bytes));
                    ui.label(format!("Blocks: {:?}", file.blocks));
                    for line in describe_metadata(file) {
                        ui.label(line);
                    }
                    ui.separator();
                    if !file.blocks.is_empty() && ui.button("Open hex viewer").clicked() {
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
        if let Some((file, _)) = flattened.get(idx) {
            let bytes = collect_block_bytes(file, &state.blocks);
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
                    ui.heading(sanitize_display(&file.path.to_string_path()));
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.monospace(format_hex(&bytes));
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

    if !state.summary_status.is_empty() {
        ui.separator();
        ui.label(&state.summary_status);
    }
}

pub fn flatten_files_tree(files: &[TapeFile], depth: usize) -> Vec<(TapeFile, usize)> {
    let mut flat = Vec::new();
    for file in files {
        flat.push((file.clone(), depth));
        if !file.children.is_empty() {
            flat.extend(flatten_files_tree(&file.children, depth + 1));
        }
    }
    flat
}

pub fn describe_metadata(file: &TapeFile) -> Vec<String> {
    match &file.metadata {
        FileMetadata::Rsx(meta) => vec![
            format!("UIC {:o},{:o}", meta.uic.0, meta.uic.1),
            format!("Protection: {:o}", meta.protection),
            format!("Directory: {}", meta.is_directory),
        ],
        FileMetadata::Rt11(meta) => vec![
            format!("Start block: {}", meta.start_block),
            format!("Length (blocks): {}", meta.length_blocks),
            format!("Extension: {}", meta.ext),
        ],
        FileMetadata::Rsts(meta) => vec![
            format!("Owner UIC {:o},{:o}", meta.owner_uic.0, meta.owner_uic.1),
            format!("Blocks: {}", meta.blocks),
            format!("Status: {:#06X}", meta.status),
        ],
        FileMetadata::Vms(meta) => vec![format!("VMS metadata placeholder: {}", meta.placeholder)],
        FileMetadata::Raw => vec!["Raw block".to_string()],
    }
}

pub fn collect_block_bytes(file: &TapeFile, blocks: &[TapeBlock]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for idx in &file.blocks {
        if let Some(block) = blocks.iter().find(|b| b.index == *idx) {
            bytes.extend_from_slice(block.raw.as_ref());
        }
    }
    bytes
}
