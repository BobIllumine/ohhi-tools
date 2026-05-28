use eframe::{egui, Frame};
use eframe::egui::Ui;
use crate::render::{ui_board_canvas, ui_solution_panel, ui_toolbar_bottom, ui_toolbar_top};
use crate::state::GuiState;

/// Top-level egui application. Owns the [`GuiState`] and divides the window
/// into three panels: a left toolbar (20–30 % width), a right solution panel
/// (20–30 %), and a center canvas that fills the remainder.
///
/// Rendering is delegated to the four functions in [`render`](crate::render).
/// All state mutation goes through [`apply`](crate::state::apply).
pub struct App {
    gui_state: Option<GuiState>
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gui_state: Some(GuiState::new((4, 4), None))
        }
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let total_width = ui.available_width();
            egui::Panel::left("toolbar")
                .min_size(total_width * 0.2)
                .max_size(total_width * 0.3)
                .show_inside(ui, |ui| {
                    egui::Panel::bottom("toolbar_bottom").show_inside(ui, |ui| {
                        ui_toolbar_bottom(&mut self.gui_state.as_mut().unwrap(), ui);
                    });
                    ui_toolbar_top(&mut self.gui_state.as_mut().unwrap(), ui);
                });
            egui::Panel::right("solution")
                .min_size(total_width * 0.2)
                .max_size(total_width * 0.3)
                .show_inside(ui, |ui| {
                    ui_solution_panel(&mut self.gui_state.as_mut().unwrap(), ui);
                });
            ui_board_canvas(&mut self.gui_state.as_mut().unwrap(), ui);
        });
    }
}