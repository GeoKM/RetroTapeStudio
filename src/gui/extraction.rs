use std::fs;
use std::path::PathBuf;

use egui::{self, ScrollArea};
use rfd::FileDialog;

use crate::backup::extract::{assemble_files, ExtractedFile};
use crate::tap::reader::TapEntry;

#[derive(Debug, Default, Clone)]
pub struct ExtractionState {
    pub output_dir: Option<PathBuf>,
    pub status: String,
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
                ui.horizontal(|ui| {
                    ui.label(&file.name);
                    ui.label(format!("blocks: {}", file.blocks.len()));
                    ui.label(format!("payload: {} bytes", file.payload().len()));
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
}

fn write_file(path: &PathBuf, file: &ExtractedFile) -> Result<(), String> {
    fs::write(path, file.payload())
        .map_err(|err| format!("failed to write {}: {}", path.display(), err))
}
