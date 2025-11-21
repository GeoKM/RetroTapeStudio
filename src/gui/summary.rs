//! Summary tab: presents aggregates and an export-to-text action for the current save-set.
use egui::{self, Grid};
use rfd::FileDialog;

use crate::summary::SaveSetSummary;

use super::state::AppState;

pub fn summary_tab(ui: &mut egui::Ui, state: &mut AppState) {
    let Some(summary) = &state.summary else {
        ui.label("No summary available");
        return;
    };

    Grid::new("summary_grid").num_columns(2).show(ui, |ui| {
        ui.label("Total files");
        ui.label(summary.total_files.to_string());
        ui.end_row();

        ui.label("Total directories");
        ui.label(summary.total_directories.to_string());
        ui.end_row();

        ui.label("Total blocks");
        ui.label(summary.total_blocks.to_string());
        ui.end_row();

        ui.label("Total bytes");
        ui.label(summary.total_bytes.to_string());
        ui.end_row();

        ui.label("Largest file");
        ui.label(summary.largest_file.clone().unwrap_or_else(|| "-".into()));
        ui.end_row();

        ui.label("Smallest file");
        ui.label(summary.smallest_file.clone().unwrap_or_else(|| "-".into()));
        ui.end_row();

        ui.label("Block efficiency");
        ui.label(format!("{:.3}", summary.block_efficiency));
        ui.end_row();

        ui.label("Log warnings");
        ui.label(summary.log_warnings.to_string());
        ui.end_row();

        ui.label("Log errors");
        ui.label(summary.log_errors.to_string());
        ui.end_row();

        ui.label("Tracks");
        ui.label(summary.tracks.clone().unwrap_or_else(|| "-".into()));
        ui.end_row();

        ui.label("Density");
        ui.label(summary.density.clone().unwrap_or_else(|| "-".into()));
        ui.end_row();

        ui.label("Blocks read");
        ui.label(summary.blocks_read.clone().unwrap_or_else(|| "-".into()));
        ui.end_row();
    });

    ui.separator();
    ui.heading("Record formats");
    for (rfm, count) in summary.rfm_hist.iter() {
        ui.label(format!("{}: {}", rfm, count));
    }

    ui.separator();
    ui.heading("Protections");
    for (prot, count) in summary.protection_hist.iter() {
        ui.label(format!("{}: {}", prot, count));
    }

    ui.separator();
    if ui.button("Export Summary TXT").clicked() {
        if let Some(path) = FileDialog::new().set_file_name("summary.txt").save_file() {
            let text = format_summary_txt(summary);
            match std::fs::write(&path, text) {
                Ok(_) => state.summary_status = format!("Saved {}", path.display()),
                Err(err) => state.summary_status = format!("Failed: {}", err),
            }
        }
    }
    ui.label(&state.summary_status);
}

fn format_summary_txt(summary: &SaveSetSummary) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Total files: {}", summary.total_files));
    lines.push(format!("Total directories: {}", summary.total_directories));
    lines.push(format!("Total blocks: {}", summary.total_blocks));
    lines.push(format!("Total bytes: {}", summary.total_bytes));
    lines.push(format!(
        "Largest file: {}",
        summary.largest_file.clone().unwrap_or_else(|| "-".into())
    ));
    lines.push(format!(
        "Smallest file: {}",
        summary.smallest_file.clone().unwrap_or_else(|| "-".into())
    ));
    lines.push(format!("Block efficiency: {:.3}", summary.block_efficiency));
    lines.push(format!("Log warnings: {}", summary.log_warnings));
    lines.push(format!("Log errors: {}", summary.log_errors));
    if let Some(t) = &summary.tracks {
        lines.push(format!("Tracks: {}", t));
    }
    if let Some(d) = &summary.density {
        lines.push(format!("Density: {}", d));
    }
    if let Some(b) = &summary.blocks_read {
        lines.push(format!("Blocks read: {}", b));
    }
    lines.push("Record formats:".into());
    for (k, v) in summary.rfm_hist.iter() {
        lines.push(format!("  {}: {}", k, v));
    }
    lines.push("Protections:".into());
    for (k, v) in summary.protection_hist.iter() {
        lines.push(format!("  {}: {}", k, v));
    }
    lines.join("\n")
}
