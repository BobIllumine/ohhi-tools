use dioxus::prelude::*;
use ohhi_app::AppState;
use ohhi_app::analysis::{AnalysisAction, Constructor, Engine, Reducer, apply};
use ohhi_solver::structs::{Technique, TechniqueSet};

use super::board::Board;

// ── Shared panel/button styles ────────────────────────────────────────────────

const PANEL_BG:  &str = "#181818";
const BORDER:    &str = "#2e2e2e";
const BTN_BG:    &str = "#2a2a2a";
const BTN_FG:    &str = "#e0e0e0";
const BTN_BORD:  &str = "#404040";
const MUTED:     &str = "#686868";
const INPUT_BG:  &str = "#1a1a1a";

// ── Root ─────────────────────────────────────────────────────────────────────

#[component]
pub fn AnalysisView(mut state: Signal<AppState>) -> Element {
    let board         = state.read().analysis.board.clone();
    let overlay       = state.read().analysis.overlay.clone();
    let overlay_step  = state.read().analysis.overlay_step;
    let show_sigs     = state.read().analysis.show_signatures;
    let has_overlay   = state.read().analysis.overlay.is_some();
    let engine_is_v1  = matches!(state.read().analysis.engine, Engine::V1(_));
    let techniques    = match state.read().analysis.engine {
        Engine::V1(ts) => ts,
        Engine::V2     => TechniqueSet::NONE,
    };
    let overlay_total = overlay.as_ref().map(|t| t.steps.len()).unwrap_or(0);

    rsx! {
        div {
            style: "display: flex; height: 100%; overflow: hidden;",

            // ── LEFT PANEL ─────────────────────────────────────────────────
            div {
                style: "
                    width: 220px; min-width: 220px;
                    background: {PANEL_BG};
                    border-right: 1px solid {BORDER};
                    display: flex; flex-direction: column; overflow: hidden;
                ",

                div {
                    style: "flex: 1; overflow-y: auto; padding: 14px 12px; display: flex; flex-direction: column; gap: 18px;",

                    // Board actions
                    Section { title: "Board",
                        div { style: "display: flex; gap: 8px;",
                            IconBtn { icon: "↺", title: "Undo",  onclick: move |_| act(state, AnalysisAction::Undo) }
                            IconBtn { icon: "↻", title: "Redo",  onclick: move |_| act(state, AnalysisAction::Redo) }
                            IconBtn { icon: "✕", title: "Clear", onclick: move |_| act(state, AnalysisAction::ClearBoard) }
                        }
                    }

                    ResizePanel { state }
                    GeneratePanel { state }
                }

                // Seed — pinned to the bottom
                div {
                    style: "border-top: 1px solid {BORDER}; padding: 12px;",
                    SeedPanel { state }
                }
            }

            // ── BOARD — centered in remaining space ────────────────────────
            div {
                style: "flex: 1; display: flex; align-items: center; justify-content: center; overflow: auto; background: #1e1e1e;",
                Board {
                    board:           board.clone(),
                    overlay:         overlay.clone(),
                    overlay_step,
                    show_signatures: show_sigs,
                    on_cell_click:   move |(r, c)| act(state, AnalysisAction::CycleCell(r, c)),
                }
            }

            // ── RIGHT PANEL ────────────────────────────────────────────────
            div {
                style: "
                    width: 220px; min-width: 220px;
                    background: {PANEL_BG};
                    border-left: 1px solid {BORDER};
                    overflow-y: auto;
                    padding: 14px 12px;
                    display: flex; flex-direction: column; gap: 18px;
                ",

                // Validate
                Section { title: "Validate",
                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        Btn { label: "Validate board", onclick: move |_| act(state, AnalysisAction::Validate) }
                        Toggle {
                            label: if show_sigs { "Hide signatures" } else { "Show signatures" },
                            active: show_sigs,
                            onclick: move |_| act(state, AnalysisAction::ToggleSignatures),
                        }
                    }
                    if let Some(v) = state.read().analysis.last_validation.clone() {
                        div {
                            style: "margin-top: 10px; padding: 8px 10px; background: #141414; border-radius: 8px; font-size: 13px; border: 1px solid {BORDER};",
                            match v {
                                Ok(()) => rsx! { span { style: "color: #5dba6a;", "✓ Valid" } },
                                Err(viol) => {
                                    let mut msgs: Vec<&str> = Vec::new();
                                    if viol.rule_of_3()          { msgs.push("Rule-of-3"); }
                                    if viol.rule_of_equity()     { msgs.push("Equity"); }
                                    if viol.rule_of_duplication(){ msgs.push("Duplicate"); }
                                    if viol.incomplete()         { msgs.push("Incomplete"); }
                                    let msg = msgs.join(", ");
                                    rsx! { span { style: "color: #e05555;", "✗ {msg}" } }
                                }
                            }
                        }
                    }
                }

                // Deduction
                Section { title: "Deduction",
                    div { style: "display: flex; gap: 0; margin-bottom: 12px;",
                        SegBtn { label: "V1", active: engine_is_v1,  pos: "left",
                            onclick: move |_| act(state, AnalysisAction::SetEngine(Engine::V1(TechniqueSet::ALL))) }
                        SegBtn { label: "V2", active: !engine_is_v1, pos: "right",
                            onclick: move |_| act(state, AnalysisAction::SetEngine(Engine::V2)) }
                    }

                    if engine_is_v1 {
                        div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;",
                            Check { label: "Pair extension",  t: Technique::PairExtension,  techniques, state }
                            Check { label: "Gap fill",        t: Technique::GapFill,         techniques, state }
                            Check { label: "Saturation",      t: Technique::Saturation,      techniques, state }
                            Check { label: "Twin completion", t: Technique::TwinCompletion,  techniques, state }
                        }
                    }

                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                        Btn { label: "Deduce", onclick: move |_| act(state, AnalysisAction::Deduce) }
                        if has_overlay {
                            Btn { label: "Apply deduction", onclick: move |_| act(state, AnalysisAction::ApplyDeduction) }
                            Btn { label: "Clear overlay",   onclick: move |_| act(state, AnalysisAction::ClearOverlay) }
                        }
                    }

                    if has_overlay {
                        div { style: "margin-top: 12px; display: flex; flex-direction: column; gap: 8px;",
                            // Step nav buttons
                            div { style: "display: flex; align-items: center; gap: 4px;",
                                StepBtn { label: "«", title: "First step",
                                    onclick: move |_| state.write().analysis.overlay_step = 0 }
                                StepBtn { label: "‹", title: "Previous step",
                                    onclick: move |_| {
                                        let s = state.read().analysis.overlay_step;
                                        state.write().analysis.overlay_step = s.saturating_sub(1);
                                    }
                                }
                                span { style: "flex: 1; text-align: center; font-size: 13px; color: {MUTED}; white-space: nowrap;",
                                    "{overlay_step} / {overlay_total}"
                                }
                                StepBtn { label: "›", title: "Next step",
                                    onclick: move |_| {
                                        let s = state.read().analysis.overlay_step;
                                        let max = state.read().analysis.overlay.as_ref().map(|t| t.steps.len()).unwrap_or(0);
                                        state.write().analysis.overlay_step = (s + 1).min(max);
                                    }
                                }
                                StepBtn { label: "»", title: "Last step",
                                    onclick: move |_| {
                                        let max = state.read().analysis.overlay.as_ref().map(|t| t.steps.len()).unwrap_or(0);
                                        state.write().analysis.overlay_step = max;
                                    }
                                }
                            }
                            // Slider for quick scrubbing
                            input {
                                r#type: "range", min: "0", max: "{overlay_total}", value: "{overlay_step}",
                                style: "width: 100%;",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<usize>() {
                                        state.write().analysis.overlay_step = v;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn act(mut state: Signal<AppState>, action: AnalysisAction) {
    let _ = apply(&mut state.write().analysis, action);
}

// ── Resize panel ──────────────────────────────────────────────────────────────

#[component]
fn ResizePanel(mut state: Signal<AppState>) -> Element {
    let cur_n = state.read().analysis.board.width();
    let mut n_buf = use_signal(move || cur_n.to_string());

    rsx! {
        Section { title: "Board size",
            div { style: "display: flex; align-items: center; gap: 8px;",
                input {
                    r#type: "number", min: "2", max: "16", step: "2",
                    value: "{n_buf}",
                    style: "width: 60px; padding: 6px 8px; background: {INPUT_BG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                    oninput: move |e| *n_buf.write() = e.value(),
                }
                span { style: "color: {MUTED}; font-size: 13px;", "× N" }
                Btn { label: "Apply",
                    onclick: move |_| {
                        if let Ok(n) = n_buf.read().trim().parse::<usize>() {
                            act(state, AnalysisAction::Resize(n, n));
                        }
                    }
                }
            }
        }
    }
}

// ── Generate panel ────────────────────────────────────────────────────────────

#[component]
fn GeneratePanel(mut state: Signal<AppState>) -> Element {
    let mut n_buf    = use_signal(|| "6".to_string());
    let mut seed_buf = use_signal(String::new);

    rsx! {
        Section { title: "Generate",
            div { style: "display: flex; flex-direction: column; gap: 10px;",

                // Size + Seed inputs
                div { style: "display: flex; flex-direction: column; gap: 6px;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        label { style: "font-size: 12px; color: {MUTED}; width: 34px; flex-shrink: 0;", "Size" }
                        input {
                            r#type: "number", min: "2", max: "16", step: "2",
                            value: "{n_buf}",
                            style: "min-width: 0; flex: 1; padding: 6px 8px; background: {INPUT_BG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                            oninput: move |e| *n_buf.write() = e.value(),
                        }
                    }
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        label { style: "font-size: 12px; color: {MUTED}; width: 34px; flex-shrink: 0;", "Seed" }
                        input {
                            r#type: "text", placeholder: "random",
                            value: "{seed_buf}",
                            // min-width: 0 prevents flex children from overflowing
                            style: "min-width: 0; flex: 1; padding: 6px 8px; background: {INPUT_BG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                            oninput: move |e| *seed_buf.write() = e.value(),
                        }
                    }
                }

                // Constructor segmented control — equal widths
                div { style: "display: flex; gap: 0;",
                    SegBtn { label: "OG",      active: state.read().analysis.gen_constructor == Constructor::Og,      pos: "left",
                        onclick: move |_| act(state, AnalysisAction::SetConstructor(Constructor::Og)) }
                    SegBtn { label: "Toolkit", active: state.read().analysis.gen_constructor == Constructor::Toolkit, pos: "right",
                        onclick: move |_| act(state, AnalysisAction::SetConstructor(Constructor::Toolkit)) }
                }

                // Reducer segmented control — equal widths
                div { style: "display: flex; gap: 0;",
                    SegBtn { label: "Breakdown", active: state.read().analysis.gen_reducer == Reducer::Breakdown, pos: "left",
                        onclick: move |_| act(state, AnalysisAction::SetReducer(Reducer::Breakdown)) }
                    SegBtn { label: "Carve",     active: state.read().analysis.gen_reducer == Reducer::Carve,     pos: "right",
                        onclick: move |_| act(state, AnalysisAction::SetReducer(Reducer::Carve)) }
                }

                if let Some(q) = state.read().analysis.last_quality {
                    p { style: "margin: 0; font-size: 12px; color: #5dba6a;", "Empty: {(q * 100.0) as u32}%" }
                }

                // Apply / Cancel
                div { style: "display: flex; gap: 8px;",
                    button {
                        style: "flex: 1; padding: 7px 0; background: #4a4a4a; color: #f0f0f0; border: none; border-radius: 6px; font-weight: 600;",
                        onclick: move |_| {
                            let n    = n_buf.read().trim().parse::<usize>().unwrap_or(6);
                            let seed = seed_buf.read().trim().parse::<u64>().ok();
                            act(state, AnalysisAction::Generate { n, seed });
                        },
                        "Apply"
                    }
                    button {
                        style: "flex: 1; padding: 7px 0; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                        onclick: move |_| {
                            *n_buf.write()    = "6".to_string();
                            *seed_buf.write() = String::new();
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}

// ── Seed I/O ──────────────────────────────────────────────────────────────────

#[component]
fn SeedPanel(mut state: Signal<AppState>) -> Element {
    let mut seed_text = use_signal(String::new);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 8px;",
            p { style: "margin: 0 0 2px; font-size: 11px; font-weight: 600; letter-spacing: .06em; text-transform: uppercase; color: {MUTED};", "Seed" }
            textarea {
                style: "width: 100%; height: 88px; background: {INPUT_BG}; color: #e0e0e0; border: 1px solid {BTN_BORD}; border-radius: 6px; padding: 7px; resize: vertical;",
                placeholder: "Paste seed here…",
                value: "{seed_text}",
                oninput: move |e| *seed_text.write() = e.value(),
            }
            if let Some(err) = error_msg.read().clone() {
                p { style: "margin: 0; font-size: 12px; color: #e05555;", "{err}" }
            }
            div { style: "display: flex; gap: 8px;",
                button {
                    style: "flex: 1; padding: 6px 0; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                    onclick: move |_| {
                        let seed = seed_text.read().clone();
                        match apply(&mut state.write().analysis, AnalysisAction::LoadSeed(seed)) {
                            Ok(())  => *error_msg.write() = None,
                            Err(e)  => *error_msg.write() = Some(e.0),
                        }
                    },
                    "Load"
                }
                button {
                    style: "flex: 1; padding: 6px 0; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                    onclick: move |_| {
                        let board = state.read().analysis.board.clone();
                        *seed_text.write() = ohhi_core::seed::encode(&board);
                    },
                    "Export"
                }
            }
        }
    }
}

// ── Section wrapper ───────────────────────────────────────────────────────────

#[component]
fn Section(title: &'static str, children: Element) -> Element {
    rsx! {
        div {
            p { style: "margin: 0 0 8px; font-size: 11px; font-weight: 600; letter-spacing: .07em; text-transform: uppercase; color: {MUTED};", "{title}" }
            {children}
        }
    }
}

// ── Step nav button ───────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct StepBtnProps {
    label:   &'static str,
    title:   &'static str,
    onclick: EventHandler<()>,
}

#[component]
fn StepBtn(props: StepBtnProps) -> Element {
    rsx! {
        button {
            title: "{props.title}",
            style: "
                width: 44px; height: 44px;
                background: {BTN_BG}; color: {BTN_FG};
                border: 1px solid {BTN_BORD}; border-radius: 8px;
                font-size: 22px; line-height: 1;
                display: flex; align-items: center; justify-content: center;
                cursor: pointer;
            ",
            onclick: move |_| props.onclick.call(()),
            "{props.label}"
        }
    }
}

// ── Icon button ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct IconBtnProps {
    icon:    &'static str,
    title:   &'static str,
    onclick: EventHandler<()>,
}

#[component]
fn IconBtn(props: IconBtnProps) -> Element {
    rsx! {
        button {
            title: "{props.title}",
            style: "width: 38px; height: 38px; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 8px; font-size: 18px; display: flex; align-items: center; justify-content: center;",
            onclick: move |_| props.onclick.call(()),
            "{props.icon}"
        }
    }
}

// ── Text button ───────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct BtnProps {
    label:   String,
    onclick: EventHandler<()>,
}

#[component]
fn Btn(props: BtnProps) -> Element {
    rsx! {
        button {
            style: "padding: 6px 12px; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px; width: 100%;",
            onclick: move |_| props.onclick.call(()),
            "{props.label}"
        }
    }
}

// ── Segmented control button (equal widths, joined borders) ───────────────────

#[derive(Props, Clone, PartialEq)]
struct SegBtnProps {
    label:   &'static str,
    active:  bool,
    /// "left" | "right" | "middle" — controls border-radius and border overlap
    pos:     &'static str,
    onclick: EventHandler<()>,
}

#[component]
fn SegBtn(props: SegBtnProps) -> Element {
    let radius = match props.pos {
        "left"   => "6px 0 0 6px",
        "right"  => "0 6px 6px 0",
        _        => "0",
    };
    let (bg, fg, fw) = if props.active {
        ("#505050", "#f0f0f0", "600")
    } else {
        (BTN_BG, "#909090", "400")
    };
    // Right segment doesn't double the shared border
    let border = if props.pos == "right" {
        format!("border: 1px solid {BTN_BORD}; border-left: none;")
    } else {
        format!("border: 1px solid {BTN_BORD};")
    };
    let style = format!(
        "flex: 1; padding: 6px 0; background: {bg}; color: {fg}; {border} \
         border-radius: {radius}; font-weight: {fw}; text-align: center;"
    );
    rsx! {
        button { style: "{style}", onclick: move |_| props.onclick.call(()), "{props.label}" }
    }
}

// ── Toggle ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ToggleProps {
    label:   String,
    active:  bool,
    onclick: EventHandler<()>,
}

#[component]
fn Toggle(props: ToggleProps) -> Element {
    let (bg, fg, fw) = if props.active { ("#505050", "#f0f0f0", "600") } else { (BTN_BG, "#909090", "400") };
    let style = format!("width: 100%; padding: 6px 12px; background: {bg}; color: {fg}; border: 1px solid {BTN_BORD}; border-radius: 6px; font-weight: {fw}; text-align: center;");
    rsx! {
        button { style: "{style}", onclick: move |_| props.onclick.call(()), "{props.label}" }
    }
}

// ── Technique checkbox ────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct CheckProps {
    label:      &'static str,
    t:          Technique,
    techniques: TechniqueSet,
    state:      Signal<AppState>,
}

#[component]
fn Check(props: CheckProps) -> Element {
    let state = props.state;
    let t = props.t;
    let checked = props.techniques.contains(t);
    rsx! {
        label {
            style: "display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; color: #c8c8c8;",
            input { r#type: "checkbox", checked, onchange: move |_| act(state, AnalysisAction::ToggleTechnique(t)) }
            "{props.label}"
        }
    }
}
