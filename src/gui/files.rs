//! Files tab: displays reconstructed file tree with metadata and hex viewers.
use egui::TextStyle;
use egui::{self, Align, Layout, ScrollArea, Vec2, Window};
use rfd::FileDialog;

use crate::core::block::TapeBlock;
use crate::core::extract::extract_file;
use crate::core::file::{FileMetadata, TapeFile};
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
                    ui.label(format!("{:>4}", idx));
                    ui.add_space(8.0);
                    ui.add_space(*depth as f32 * 10.0);
                    let icon = if file.children.is_empty() {
                        "\u{1F4C4}"
                    } else {
                        "\u{1F4C1}"
                    };
                    ui.label(icon);
                    ui.add_space(4.0);
                    ui.label(sanitize_display(&file.path.to_string_path()));
                    ui.add_space(8.0);
                    ui.label(format!("{:?}", file.format));
                    ui.add_space(8.0);
                    ui.label(format!("{} bytes", file.size_bytes));
                    ui.add_space(8.0);
                    ui.label(format!("{} blk", file.blocks.len()));
                    ui.add_space(8.0);
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
                    ui.label(format!(
                        "Path: {}",
                        sanitize_display(&file.path.to_string_path())
                    ));
                    ui.label(format!("Format: {:?}", file.format));
                    ui.label(format!("Size: {} bytes", file.size_bytes));
                    ui.label(format!("Blocks: {}", file.blocks.len()));
                    ui.separator();
                    ui.label("Metadata:");
                    for line in describe_metadata(file) {
                        ui.label(line);
                    }
                    ui.separator();
                    if !file.blocks.is_empty() && ui.button("Open Hex Viewer").clicked() {
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
                            ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                            let mut line = 0usize;
                            let mut pos = 0usize;
                            while pos < bytes.len() {
                                let chunk = &bytes[pos..usize::min(pos + 16, bytes.len())];
                                let mut hex_part = String::new();
                                let mut ascii_part = String::new();
                                for b in chunk {
                                    hex_part.push_str(&format!("{:02X} ", b));
                                    let c = char::from(*b);
                                    if c.is_ascii_graphic() || c == ' ' {
                                        ascii_part.push(c);
                                    } else {
                                        ascii_part.push('.');
                                    }
                                }
                                ui.label(format!(
                                    "{:08X}:  {:<48}  |{}|",
                                    line * 16,
                                    hex_part,
                                    ascii_part
                                ));
                                pos += 16;
                                line += 1;
                            }
                            ui.style_mut().override_text_style = None;
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
            format!("Directory: {}", meta.is_directory),
            format!("UIC {:03o},{:03o}", meta.uic.0, meta.uic.1),
            format!("Protection: {:o}", meta.protection),
        ],
        FileMetadata::Rt11(meta) => vec![
            format!(
                "File: {}",
                sanitize_display(&if meta.ext.is_empty() {
                    file.path.to_string_path()
                } else {
                    format!("{}.{}", file.path.to_string_path(), meta.ext)
                })
            ),
            format!("Start block: {}", meta.start_block),
            format!("Length (blocks): {}", meta.length_blocks),
        ],
        FileMetadata::Rsts(meta) => vec![
            format!(
                "Owner UIC {:03o},{:03o}",
                meta.owner_uic.0, meta.owner_uic.1
            ),
            format!("Status: 0x{:04X}", meta.status),
            format!("Blocks: {}", meta.blocks),
        ],
        FileMetadata::Vms(meta) => {
            let mut out = Vec::new();
            out.push(format!(
                "File ID: ({}, {}, {})",
                meta.file_id.0, meta.file_id.1, meta.file_id.2
            ));
            out.push(format!("Revision: {}", meta.rev));
            out.push(format!("Sequence: {}", meta.seq));
            out.push(format!(
                "Owner UIC: {:o},{:o}",
                meta.owner_uic.0, meta.owner_uic.1
            ));
            out.push(format!("Protection: {:o}", meta.protection));
            out.push(format!(
                "Record Format: {}",
                describe_record_format(meta.record_format)
            ));
            out.push(format!(
                "Record Attributes: 0x{:02X}",
                meta.record_attributes
            ));
            out.push(format!("Record Length: {}", meta.record_length));
            out.push(format!("File Type: {}", meta.file_type));
            out.push(format!("Backup Flags: 0x{:04X}", meta.backup_flags));
            if let Some(t) = &meta.creation_time {
                out.push(format!("Created: {}", t));
            }
            if let Some(t) = &meta.revision_time {
                out.push(format!("Revised: {}", t));
            }
            if let Some(t) = &meta.expiration_time {
                out.push(format!("Expires: {}", t));
            }
            out
        }
        FileMetadata::Raw => vec!["Raw data, no metadata".to_string()],
    }
}

fn describe_record_format(rfm: u8) -> &'static str {
    match rfm {
        0 => "Undefined",
        1 => "Fixed",
        2 => "Variable",
        3 => "VFC",
        _ => "Unknown",
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
