//! Extraction tab: lets users choose an output folder and write assembled file payloads to disk.
use egui::{self, ScrollArea, Window};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

use crate::backup::extract::{assemble_files, ExtractedFile};
use crate::utils::hex::format_hex;
use crate::tap::reader::TapEntry;

#[derive(Debug, Default, Clone)]
pub struct ExtractionState {
    pub output_dir: Option<PathBuf>,
    pub status: String,
    pub hex_view: Option<(String, Vec<u8>)>,
}

/// Render the Extraction tab UI.
pub fn extraction_tab(ui: &mut egui::Ui, entries: &[TapEntry], state: &mut ExtractionState) {
    let files = assemble_files(entries);

    ui.heading("Extracted Files");
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for file in &files {
                let payload = file.payload();
                let preview = text_preview(&payload);
                let block_count = file.blocks.len();
                let payload_len = payload.len();
                ui.horizontal(|ui| {
                    ui.label(&file.name);
                    ui.label(format!("blocks: {}", block_count));
                    ui.label(format!("payload: {} bytes", payload_len));
                    ui.label(format!("type: {}", record_type_label(file)));
                    ui.label(preview);
                    if ui.button("View details").clicked() {
                        state.hex_view = Some((file.name.clone(), payload.clone()));
                    }
                });
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
        Window::new("Hex Viewer")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.heading(&name);
                ui.monospace(format_hex(&bytes));
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            state.hex_view = None;
        }
    }
}

fn write_file(path: &PathBuf, file: &ExtractedFile) -> Result<(), String> {
    fs::write(path, file.payload())
        .map_err(|err| format!("failed to write {}: {}", path.display(), err))
}

fn text_preview(payload: &[u8]) -> String {
    std::str::from_utf8(payload)
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

fn record_type_label(file: &ExtractedFile) -> &'static str {
    if file.blocks.is_empty() {
        "Unknown"
    } else {
        "VMS BACKUP block"
    }
}
