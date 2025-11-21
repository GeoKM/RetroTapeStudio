//! App shell: wires together all tabs and routes shared state.
use egui::{self, CentralPanel};

use super::contents::contents_table;
use super::extraction::extraction_tab;
use super::files::files_tab;
use super::input::input_tab;
use super::logview::draw_log;
use super::state::{AppState, MainTab};
use super::summary::summary_tab;

/// Simple top-level tab rendering that places Summary after Files.
pub fn render_app(ctx: &egui::Context, state: &mut AppState) {
    CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            tab_button(ui, "Input", MainTab::Input, state);
            tab_button(ui, "Contents", MainTab::Contents, state);
            tab_button(ui, "Extraction", MainTab::Extraction, state);
            tab_button(ui, "Files", MainTab::Files, state);
            tab_button(ui, "Summary", MainTab::Summary, state);
            tab_button(ui, "Log", MainTab::Log, state);
        });
        ui.separator();
        match state.current_tab {
            MainTab::Input => input_tab(ui, state),
            MainTab::Contents => {
                let entries = state.tap_state.entries.clone();
                contents_table(ui, &entries, state)
            }
            MainTab::Extraction => {
                let entries = state.tap_state.entries.clone();
                extraction_tab(ui, &entries, &mut state.extraction)
            }
            MainTab::Files => files_tab(ui, state),
            MainTab::Summary => summary_tab(ui, state),
            MainTab::Log => draw_log(ui, &state.log_state.data),
        }
    });
}

fn tab_button(ui: &mut egui::Ui, label: &str, tab: MainTab, state: &mut AppState) {
    let selected = state.current_tab == tab;
    if ui.selectable_label(selected, label).clicked() {
        state.current_tab = tab;
    }
}
