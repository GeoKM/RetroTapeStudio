//! Extraction tab: lets users choose an output folder and write assembled file payloads to disk.
use egui::{self, Align, Layout, ScrollArea, Vec2, Window};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

use crate::backup::extract::{assemble_files, assemble_vms_files, ExtractedFile, VmsFile};
use crate::utils::hex::format_hex;
use crate::utils::text::{is_mostly_text, sanitize_display};
use crate::tap::reader::TapEntry;

#[derive(Debug, Default, Clone)]
pub struct ExtractionState {
    pub output_dir: Option<PathBuf>,
    pub status: String,
    pub hex_view: Option<(String, Vec<u8>)>,
}

/// Render the Extraction tab UI.
pub fn extraction_tab(ui: &mut egui::Ui, entries: &[TapEntry], state: &mut ExtractionState) {
    let vms_files = assemble_vms_files(entries);
    let files = assemble_files(entries);

    ui.heading("Extracted Files");
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if !vms_files.is_empty() {
                for file in &vms_files {
                    let payload_len: usize = file.blocks.iter().map(|b| b.payload.len()).sum();
                    let sanitized_name = sanitize_display(&format!(
                        "{};{}",
                        file.headers.full_name(),
                        file.headers.version
                    ));
                    let text_hint = if is_mostly_text(&collect_vms_prefix(file, 512)) {
                        "(text)"
                    } else {
                        "(binary)"
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.label(sanitized_name);
                        ui.label(format!("blocks: {}", file.blocks.len()));
                        ui.label(format!("payload: {} bytes", payload_len));
                        ui.label(format!(
                            "type: {}",
                            record_type_label(&file.blocks, !file.headers.file_type.is_empty())
                        ));
                        ui.label(format!("UIC {:X}", file.headers.owner_uic));
                        ui.label(text_hint);
                        if ui.button("View details").clicked() {
                            let name_with_version =
                                format!("{};{}", file.headers.full_name(), file.headers.version);
                            state
                                .hex_view
                                .replace((name_with_version, collect_vms_all(file)));
                        }
                    });
                }
            } else {
                for file in &files {
                    let payload_len: usize = file.blocks.iter().map(|b| b.payload.len()).sum();
                    let sanitized_name = sanitize_display(&file.name);
                    let text_hint = if is_mostly_text(&collect_prefix(file, 512)) {
                        "(text)"
                    } else {
                        "(binary)"
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.label(sanitized_name);
                        ui.label(format!("blocks: {}", file.blocks.len()));
                        ui.label(format!("payload: {} bytes", payload_len));
                        ui.label(format!("type: {}", record_type_label(&file.blocks, false)));
                        ui.label(text_hint);
                        if ui.button("View details").clicked() {
                            state.hex_view = Some((file.name.clone(), collect_all(file)));
                        }
                    });
                }
            }
        });

    ui.separator();

    if ui.button("Choose Output Directory").clicked() {
        if let Some(path) = FileDialog::new().pick_folder() {
            state.status = format!("Output directory set to {}", path.display());
            state.output_dir = Some(path);
        }
    }

    if ui.button("Extract Files").clicked() {
        match &state.output_dir {
            Some(dir) => {
                let mut errors = Vec::new();
                for file in &files {
                    let path = dir.join(&file.name);
                    if let Err(err) = write_file(&path, file) {
                        errors.push(format!("{}: {}", file.name, err));
                    }
                }

                if errors.is_empty() {
                    state.status = format!("Extracted {} file(s)", files.len());
                } else {
                    state.status = format!("Errors: {}", errors.join("; "));
                }
            }
            None => {
                state.status = "Choose an output directory first".to_string();
            }
        }
    }

    ui.separator();
    ui.label(&state.status);

    if let Some((name, bytes)) = state.hex_view.clone() {
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
                ui.heading(sanitize_display(&name));
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
            state.hex_view = None;
        }
    }
}

fn write_file(path: &PathBuf, file: &ExtractedFile) -> Result<(), String> {
    fs::write(path, file.payload())
        .map_err(|err| format!("failed to write {}: {}", path.display(), err))
}

fn record_type_label(blocks: &[crate::backup::vms::BackupBlock], has_header: bool) -> &'static str {
    if blocks.is_empty() {
        "Unknown"
    } else {
        if has_header {
            "VMS BACKUP file"
        } else {
            "VMS BACKUP block"
        }
    }
}

fn collect_prefix(file: &ExtractedFile, limit: usize) -> Vec<u8> {
    let mut data = Vec::new();
    for block in &file.blocks {
        for byte in &block.payload {
            if data.len() >= limit {
                return data;
            }
            data.push(*byte);
        }
        if data.len() >= limit {
            break;
        }
    }
    data
}

fn collect_all(file: &ExtractedFile) -> Vec<u8> {
    let mut data = Vec::new();
    for block in &file.blocks {
        data.extend_from_slice(&block.payload);
    }
    data
}

fn collect_vms_prefix(file: &VmsFile, limit: usize) -> Vec<u8> {
    let mut data = Vec::new();
    for block in &file.blocks {
        for byte in &block.payload {
            if data.len() >= limit {
                return data;
            }
            data.push(*byte);
        }
        if data.len() >= limit {
            break;
        }
    }
    data
}

fn collect_vms_all(file: &VmsFile) -> Vec<u8> {
    let mut data = Vec::new();
    for block in &file.blocks {
        data.extend_from_slice(&block.payload);
    }
    data
}
