//! Immediate-mode rendering functions for the three UI panels.
//!
//! Each function receives `&mut GuiState` (to read board state and fire
//! [`Action`](crate::state::Action)s via [`apply`]) and `&mut egui::Ui`.
//!
//! # Panels
//!
//! - [`ui_board_canvas`] — center panel. Draws every cell as a rounded rect
//!   colored Red / Blue / dark-gray. Clicking a cell fires `CycleCell`.
//!   When signatures are enabled, row signatures are rendered to the left of
//!   the grid (binary strings, LSB-first = leftmost column) and column
//!   signatures are stacked vertically below each column.
//!   When a [`DeductionTrace`](ohhi_solver::structs::DeductionTrace) overlay is
//!   active, forced cells up to the scrub slider's position are drawn dimmed
//!   with a thick technique-colored border and a letter label:
//!   `P` = PairExtension (green), `G` = GapFill (yellow),
//!   `S` = Saturation (cyan), `T` = TwinCompletion (magenta).
//!
//! - [`ui_toolbar_top`] — left panel, top half. Undo / Redo / Clear / Resize
//!   buttons. Resize opens an inline dialog with width and height text fields.
//!
//! - [`ui_toolbar_bottom`] — left panel, bottom half. Load Seed button. Opens
//!   a multiline text-edit dialog; Apply parses the seed string.
//!
//! - [`ui_solution_panel`] — right panel. Validate button + result breakdown
//!   (lists which rules are violated). Filter-rules dialog to toggle which
//!   rules are checked. Deduction section: technique checkboxes, Deduce /
//!   Apply trace / Clear overlay buttons, step count + stalled/completed
//!   status, per-technique tally, and the step scrub slider.

use eframe::egui;
use crate::state::{apply, Constructor, GuiState, Reducer};
use ohhi_core::board::{Cell, BOARD_MAX_SIZE};
use ohhi_core::stats::NumTransforms;
use ohhi_solver::v1::deduction::Technique;
const PADDING: f32 = 0.7;

fn technique_color(t: Technique) -> egui::Color32 {
    match t {
        Technique::PairExtension => egui::Color32::from_rgb(80, 220, 80),
        Technique::GapFill => egui::Color32::from_rgb(240, 220, 70),
        Technique::Saturation => egui::Color32::from_rgb(70, 220, 240),
        Technique::TwinCompletion => egui::Color32::from_rgb(240, 90, 240),
    }
}

fn dim_cell_color(cell: Cell) -> egui::Color32 {
    match cell {
        Cell::Red => egui::Color32::from_rgb(140, 55, 55),
        Cell::Blue => egui::Color32::from_rgb(55, 80, 140),
        Cell::Nothing => egui::Color32::from_gray(40),
    }
}

fn technique_label(t: Technique) -> &'static str {
    match t {
        Technique::PairExtension => "P",
        Technique::GapFill => "G",
        Technique::Saturation => "S",
        Technique::TwinCompletion => "T",
    }
}
pub(crate) fn ui_board_canvas(state: &mut GuiState, ui: &mut egui::Ui) {
    let (dim_x, dim_y) = state.dims();

    let (rect, _resp) = ui.allocate_exact_size(
        ui.available_size(),
        egui::Sense::hover()
    );
    // When signatures are shown they need space: left for row sigs, bottom for col sigs.
    // Col sig stack height = dim_y chars × char_h = dim_y × font_size × 1.3, font_size ≈ cell_size × 0.28
    // → col factor ≈ 1 + 0.364 = 1.364
    // Row sig label width = dim_x chars × char_w = dim_x × font_size × 0.65
    // → row factor ≈ 1 + 0.182 = 1.182
    let show_sigs = state.show_signatures();
    let (col_factor, row_factor) = if show_sigs { (1.364_f32, 1.182_f32) } else { (1.0, 1.0) };

    let cell_size = (rect.width() * PADDING / (dim_x as f32 * row_factor))
        .min(rect.height() * PADDING / (dim_y as f32 * col_factor))
        .floor()
        .max(1.0);

    let sig_font_size = (cell_size * 0.28).max(9.0);
    let sig_font = egui::FontId::monospace(sig_font_size);
    let sig_color = egui::Color32::from_gray(180);

    let board_size = egui::Vec2::new(dim_x as f32 * cell_size, dim_y as f32 * cell_size);
    let left_sig_w = if show_sigs { sig_font_size * 0.65 * dim_x as f32 } else { 0.0 };
    let bot_sig_h  = if show_sigs { sig_font_size * 1.3  * dim_y as f32 } else { 0.0 };
    let total_size = board_size + egui::Vec2::new(left_sig_w, bot_sig_h);
    let layout_tl  = rect.center() - total_size * 0.5;
    let origin = egui::pos2(layout_tl.x + left_sig_w, layout_tl.y);

    let painter = ui.painter_at(rect);
    let mut hovered: Option<(usize, usize)> = None;
    for r in 0..dim_y {
        for c in 0..dim_x {
            let cell_rect = egui::Rect::from_min_size(
                origin + egui::Vec2::new(cell_size * c as f32, cell_size * r as f32),
                egui::Vec2::new(cell_size, cell_size)
            );
            let color = match state.board().get((r, c)) {
                ohhi_core::board::Cell::Red => egui::Color32::from_rgb(220, 70, 70),
                ohhi_core::board::Cell::Blue => egui::Color32::from_rgb(70, 120, 220),
                ohhi_core::board::Cell::Nothing => egui::Color32::from_gray(40),
            };
            painter.rect(
                cell_rect,
                cell_size / 9.0,
                color,
                egui::Stroke::new(
                    cell_size / 36.0,
                    color + egui::Color32::from_gray(10)
                ),
                egui::StrokeKind::Inside
            );
            let id = ui.id().with((r, c));
            let resp = ui.interact(cell_rect, id, egui::Sense::click());
            if resp.clicked() {
                let _ = apply(state, crate::state::Action::CycleCell(r, c));
            }

            if resp.hovered() {
                hovered = Some((r, c));
            }
        }
    }
    if show_sigs {
        for r in 0..dim_y {
            let sig = state.board().signature_x(&r);
            let label = format!("{:0>width$b}", sig, width = dim_x);
            let pos = origin + egui::Vec2::new(-cell_size * 0.25, cell_size * (r as f32 + 0.5));
            painter.text(pos, egui::Align2::RIGHT_CENTER, label, sig_font.clone(), sig_color);
        }
        for c in 0..dim_x {
            let sig = state.board().signature_y(&c);
            let label = format!("{:0>height$b}", sig, height = dim_y);
            let col_center_x = origin.x + cell_size * (c as f32 + 0.5);
            let char_h = sig_font.size * 1.3;
            for (i, ch) in label.chars().enumerate() {
                let pos = egui::pos2(
                    col_center_x,
                    origin.y + cell_size * dim_y as f32 + cell_size * 0.15 + char_h * i as f32,
                );
                painter.text(pos, egui::Align2::CENTER_TOP, ch.to_string(), sig_font.clone(), sig_color);
            }
        }
    }
    if let Some(trace) = state.overlay() {
        let upto = state.overlay_step().min(trace.get_steps().len());
        for step in &trace.get_steps()[..upto] {
            let (r, c) = step.at;
            let cell_rect = egui::Rect::from_min_size(
                origin + egui::Vec2::new(cell_size * c as f32, cell_size * r as f32),
                egui::Vec2::new(cell_size, cell_size)
            );
            let tech = technique_color(step.technique);
            painter.rect(
                cell_rect,
                cell_size / 9.0,
                dim_cell_color(step.cell),
                egui::Stroke::new(cell_size / 10.0, tech),
                egui::StrokeKind::Inside,
            );
            painter.text(
                cell_rect.left_top() + egui::Vec2::new(cell_size * 0.12, cell_size * 0.05),
                egui::Align2::LEFT_TOP,
                technique_label(step.technique),
                egui::FontId::monospace(cell_size * 0.3),
                tech,
            );
        }
    }
    if let Some((r, c)) = hovered {
        let rect = egui::Rect::from_min_size(
            origin + egui::Vec2::new(cell_size * c as f32, cell_size * r as f32),
            egui::Vec2::new(cell_size, cell_size)
        );
        painter.rect_stroke(
            rect, cell_size / 9.0,
            egui::Stroke::new(cell_size / 36.0, egui::Color32::from_rgb(200, 200, 200)),
            egui::StrokeKind::Outside
        );
    }
}


pub(crate) fn ui_toolbar_top(state: &mut GuiState, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        let _height = ui.available_height();
        let width = ui.available_width();
        let elem_size = 40.0;
        ui.heading("Toolbar");
        ui.separator();
        ui.add_space(elem_size * 0.2);
        let btn = |label: &str| egui::Button::new(label).min_size(egui::Vec2::new(width * 0.7, elem_size));
        let space = |ui: &mut egui::Ui| ui.add_space(elem_size * 0.1);
        if ui.add(btn("Undo")).clicked() {
            let _ = apply(state, crate::state::Action::Undo);
        }
        space(ui);
        if ui.add(btn("Redo")).clicked() {
            let _ = apply(state, crate::state::Action::Redo);
        }
        space(ui);
        if ui.add(btn("Clear")).clicked() {
            let _ = apply(state, crate::state::Action::ClearBoard);
        }
        space(ui);
        if ui.add(btn("Resize")).clicked() {
            state.dialogs().resize.open = !state.dialogs().resize.open;
            state.dialogs().resize.w_buf = format!("{}", state.board().width());
            state.dialogs().resize.h_buf = format!("{}", state.board().height());
        }
        if state.dialogs().resize.open {
            ui.add_space(elem_size * 0.2);
            ui.vertical_centered(|ui| {
                ui.label("Width");
                ui.add(egui::TextEdit::singleline(&mut state.dialogs().resize.w_buf)
                    .desired_width(width * 0.7)
                    .desired_rows(1)
                );
                ui.label("Height");
                ui.add(egui::TextEdit::singleline(&mut state.dialogs().resize.h_buf)
                    .desired_width(width * 0.7)
                    .desired_rows(1)
                );
            });
            space(ui);
            ui.vertical_centered(|ui| {
                if ui.add(btn("Apply")).clicked() {
                    if let (Ok(w), Ok(h)) = (
                        state.dialogs().resize.w_buf.parse::<usize>(),
                        state.dialogs().resize.h_buf.parse::<usize>()
                    ) {
                        if w <= BOARD_MAX_SIZE as usize && w > 0 && h <= BOARD_MAX_SIZE as usize && h > 0 {
                            let _ = apply(state, crate::state::Action::Resize(w, h));
                            state.dialogs().resize.open = false;
                            state.dialogs().resize.w_buf = String::new();
                            state.dialogs().resize.h_buf = String::new();
                        }
                    }
                }
                if ui.add(btn("Cancel")).clicked() {
                    state.dialogs().resize.open = false;
                    state.dialogs().resize.w_buf = String::new();
                    state.dialogs().resize.h_buf = String::new();
                }
            });
        }
    });
}

pub(crate) fn ui_toolbar_bottom(state: &mut GuiState, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let _height = ui.available_height();
        let width = ui.available_width();
        let elem_size = 40.0;
        let btn = |label: &str| egui::Button::new(label).min_size(egui::Vec2::new(width * 0.7, elem_size));
        let space = |ui: &mut egui::Ui| ui.add_space(elem_size * 0.1);
        ui.add_space(elem_size * 0.2);
        if ui.add(btn("Load Seed")).clicked() {
            state.dialogs().load_seed.open = !state.dialogs().load_seed.open;
            state.dialogs().load_seed.seed = String::new();
        }
        space(ui);
        if state.dialogs().load_seed.open {
            ui.vertical_centered(|ui| {
                ui.label("Seed");
                ui.add(
                    egui::TextEdit::multiline(&mut state.dialogs().load_seed.seed)
                        .desired_width(width * 0.7)
                        .desired_rows(4)
                        .font(egui::TextStyle::Monospace)
                );
            });
            ui.vertical_centered(|ui| {
                space(ui);
                if ui.add(btn("Apply")).clicked() {
                    state.dialogs().load_seed.open = false;
                    let seed = state.dialogs().load_seed.seed.clone();
                    match apply(state, crate::state::Action::LoadSeed(seed)) {
                        Ok(()) => {
                            state.dialogs().load_seed.seed.clear();
                            state.dialogs().load_seed.open = false;
                            state.dialogs().load_seed.error = None;
                        }
                        Err(e) => {
                            state.dialogs().load_seed.error = Some(e.0);
                        }
                    }
                }
                space(ui);
                if ui.add(btn("Cancel")).clicked() {
                    state.dialogs().load_seed.open = false;
                    state.dialogs().load_seed.seed = String::new();
                }
            });
        }
        if let Some(text) = state.dialogs().load_seed.error.as_deref() {
            space(ui);
            ui.label(egui::RichText::new(text.to_string()).color(egui::Color32::from_rgb(255, 100, 100)));
        }
        space(ui);
        if ui.add(btn("Export Seed")).clicked() {
            let encoded = crate::seed::encode(state.board());
            state.dialogs().export.seed = encoded;
            state.dialogs().export.open = !state.dialogs().export.open;
        }
        if state.dialogs().export.open {
            space(ui);
            ui.vertical_centered(|ui| {
                // Read-only text area showing the encoded seed.
                let mut seed = state.dialogs().export.seed.clone();
                ui.add(
                    egui::TextEdit::multiline(&mut seed)
                        .desired_width(width * 0.7)
                        .desired_rows(4)
                        .font(egui::TextStyle::Monospace)
                        .interactive(false)
                );
            });
            space(ui);
            ui.vertical_centered(|ui| {
                if ui.add(btn("Copy to clipboard")).clicked() {
                    ui.ctx().copy_text(state.dialogs().export.seed.clone());
                }
                space(ui);
                if ui.add(btn("Close")).clicked() {
                    state.dialogs().export.open = false;
                }
            });
        }

        space(ui);
        if ui.add(btn("Generate")).clicked() {
            state.dialogs().generate.open = !state.dialogs().generate.open;
        }
        if state.dialogs().generate.open {
            space(ui);
            ui.vertical_centered(|ui| {
                ui.label("Board size N");
                ui.add(
                    egui::TextEdit::singleline(&mut state.dialogs().generate.size_buf)
                        .desired_width(width * 0.4)
                );
                ui.label("RNG seed (blank = random)");
                ui.add(
                    egui::TextEdit::singleline(&mut state.dialogs().generate.seed_buf)
                        .desired_width(width * 0.4)
                );
            });
            space(ui);
            ui.vertical_centered(|ui| {
                ui.label("Constructor");
                ui.radio_value(&mut state.dialogs().generate.constructor, Constructor::Og, "OG (game-faithful)");
                ui.radio_value(&mut state.dialogs().generate.constructor, Constructor::Toolkit, "Ours (sandbox)");
                space(ui);
                ui.label("Reducer");
                ui.radio_value(&mut state.dialogs().generate.reducer, Reducer::Breakdown, "Breakdown (deduction-only)");
                ui.radio_value(&mut state.dialogs().generate.reducer, Reducer::Carve, "Carve (uniqueness-minimal)");
            });
            space(ui);
            ui.vertical_centered(|ui| {
                if ui.add(btn("Generate!")).clicked() {
                    let n_result: Result<usize, _> = state.dialogs().generate.size_buf.parse();
                    let seed_opt: Option<u64> = if state.dialogs().generate.seed_buf.is_empty() {
                        None
                    } else {
                        state.dialogs().generate.seed_buf.parse().ok()
                    };
                    match n_result {
                        Ok(n) => {
                            let constructor = state.dialogs().generate.constructor;
                            let reducer = state.dialogs().generate.reducer;
                            match apply(state, crate::state::Action::Generate { n, constructor, reducer, seed: seed_opt }) {
                                Ok(()) => {
                                    state.dialogs().generate.open = false;
                                    state.dialogs().generate.error = None;
                                }
                                Err(e) => {
                                    state.dialogs().generate.error = Some(e.0);
                                }
                            }
                        }
                        Err(_) => {
                            state.dialogs().generate.error = Some("N must be a number".to_string());
                        }
                    }
                }
                space(ui);
                if ui.add(btn("Cancel")).clicked() {
                    state.dialogs().generate.open = false;
                    state.dialogs().generate.error = None;
                }
            });
            if let Some(q) = state.dialogs().generate.last_quality {
                space(ui);
                ui.label(egui::RichText::new(format!("Last: {:.0}% empty", q * 100.0))
                    .color(egui::Color32::from_gray(180)));
            }
            if let Some(err) = state.dialogs().generate.error.clone() {
                space(ui);
                ui.label(egui::RichText::new(err).color(egui::Color32::from_rgb(255, 100, 100)));
            }
        }
    });
}

pub(crate) fn ui_solution_panel(state: &mut GuiState, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        let _height = ui.available_height();
        let width = ui.available_width();
        let elem_size = 40.0;
        ui.heading("Solutions");
        ui.add_space(elem_size * 0.2);
        ui.separator();
        let btn = |label: &str| egui::Button::new(label).min_size(egui::Vec2::new(width * 0.7, elem_size));
        let space = |ui: &mut egui::Ui| ui.add_space(elem_size * 0.1);
        space(ui);
        let sig_label = if state.show_signatures() { "Hide signatures" } else { "Show signatures" };
        if ui.add(btn(sig_label)).clicked() {
            let _ = apply(state, crate::state::Action::ToggleSignatures);
        }
        space(ui);
        if ui.add(btn("Validate")).clicked() {
            let _ = apply(state, crate::state::Action::Validate);
        }
        if let Some(result) = state.last_validation() {
            space(ui);
            match result {
                Ok(()) => {
                    ui.label(egui::RichText::new("Valid").color(egui::Color32::from_rgb(100, 220, 100)));
                }
                Err(v) => {
                    ui.label(egui::RichText::new("Invalid").color(egui::Color32::from_rgb(255, 100, 100)));
                    if v.rule_of_3() {
                        ui.label(egui::RichText::new("  \u{2022} rule of 3").color(egui::Color32::from_rgb(255, 150, 150)));
                    }
                    if v.rule_of_equity() {
                        ui.label(egui::RichText::new("  \u{2022} equity").color(egui::Color32::from_rgb(255, 150, 150)));
                    }
                    if v.rule_of_duplication() {
                        ui.label(egui::RichText::new("  \u{2022} duplication").color(egui::Color32::from_rgb(255, 150, 150)));
                    }
                    if v.incomplete() {
                        ui.label(egui::RichText::new("  \u{2022} incomplete").color(egui::Color32::from_rgb(200, 200, 100)));
                    }
                }
            }
        }
        space(ui);
        if ui.add(btn("Filter rules")).clicked() {
            state.dialogs().filter_rules.open = !state.dialogs().filter_rules.open;
        }
        if state.dialogs().filter_rules.open {
            ui.vertical_centered(|ui| {
                ui.label("Rule of 3");
                ui.checkbox(&mut state.dialogs().filter_rules.filter.rule_of_3, "Enabled");
                ui.label("Rule of equity");
                ui.checkbox(&mut state.dialogs().filter_rules.filter.rule_of_equity, "Enabled");
                ui.label("Rule of duplication");
                ui.checkbox(&mut state.dialogs().filter_rules.filter.rule_of_duplication, "Enabled");
                ui.label("Incomplete");
                ui.checkbox(&mut state.dialogs().filter_rules.filter.incomplete, "Enabled");
            });
        }

        if state.last_solve().is_some() {
            space(ui);
            if ui.add(btn("Reveal solution")).clicked() {
                let _ = apply(state, crate::state::Action::RevealSolution);
            }
        }
        space(ui);
        ui.separator();
        ui.heading("Deduction");
        space(ui);

        let techs = state.techniques();
        let mut toggled: Option<Technique> = None;
        ui.vertical_centered(|ui| {
            for (t, label) in [
                (Technique::PairExtension, "Pair extension"),
                (Technique::GapFill, "Gap fill"),
                (Technique::Saturation, "Saturation"),
                (Technique::TwinCompletion, "Twin completion"),
            ] {
                let mut on = techs.contains(t);
                if ui.checkbox(&mut on, label).changed() {
                    toggled = Some(t);
                }
            }
        });
        if let Some(t) = toggled {
            let _ = apply(state, crate::state::Action::ToggleTechnique(t));
        }
        space(ui);

        if ui.add(btn("Deduce")).clicked() {
            let _ = apply(state, crate::state::Action::Deduce);
        }
        space(ui);
        if ui.add(btn("Apply trace")).clicked() {
            let _ = apply(state, crate::state::Action::ApplyDeduction);
        }
        space(ui);
        if ui.add(btn("Clear overlay")).clicked() {
            let _ = apply(state, crate::state::Action::ClearOverlay);
        }

        let summary = state.overlay().map(|trace| {
            let steps = trace.get_steps();
            let mut tally = [0usize; 4];
            for s in steps {
                tally[s.technique as usize] += 1;
            }
            (steps.len(), trace.stalled, tally)
        });
        if let Some((total, stalled, tally)) = summary {
            space(ui);
            ui.separator();
            ui.label(format!("Forced: {}", total));
            if stalled {
                ui.label(egui::RichText::new("stalled").color(egui::Color32::from_rgb(255, 120, 120)));
            } else {
                ui.label(egui::RichText::new("completed").color(egui::Color32::from_rgb(120, 220, 120)));
            }
            ui.label(format!("Pair ext: {}", tally[Technique::PairExtension as usize]));
            ui.label(format!("Gap fill: {}", tally[Technique::GapFill as usize]));
            ui.label(format!("Saturation: {}", tally[Technique::Saturation as usize]));
            ui.label(format!("Twin: {}", tally[Technique::TwinCompletion as usize]));
            space(ui);
            ui.horizontal(|ui| {
                if ui.button("‹").clicked() && *state.overlay_step_mut() > 0 {
                    *state.overlay_step_mut() -= 1;
                }
                ui.add(egui::Slider::new(state.overlay_step_mut(), 0..=total).text("step"));
                if ui.button("›").clicked() && *state.overlay_step_mut() < total {
                    *state.overlay_step_mut() += 1;
                }
            });
        }
    });
}
