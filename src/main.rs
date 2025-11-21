//! Application entrypoint: launches the egui UI using eframe.
use eframe::{egui, NativeOptions};

use retro_tape_studio_v6_safe::gui::app::render_app;
use retro_tape_studio_v6_safe::gui::state::AppState;

struct RetroTapeApp {
    state: AppState,
}

impl RetroTapeApp {
    fn new() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for RetroTapeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        render_app(ctx, &mut self.state);
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "RetroTapeStudio",
        options,
        Box::new(|_cc| Box::new(RetroTapeApp::new())),
    )
}
