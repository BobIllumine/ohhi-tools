use dioxus::prelude::*;
use ohhi_app::AppState;
use ohhi_app::play::{GenMode, PlayAction, apply};
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

use super::board::Board;

const PANEL_BG: &str = "#181818";
const BORDER:   &str = "#2e2e2e";
const BTN_BG:   &str = "#2a2a2a";
const BTN_FG:   &str = "#e0e0e0";
const BTN_BORD: &str = "#404040";
const MUTED:    &str = "#686868";
const INPUT_BG: &str = "#1a1a1a";

#[component]
pub fn PlayView(mut state: Signal<AppState>) -> Element {
    // Background timer: tick every 100 ms (desktop only).
    #[cfg(not(target_arch = "wasm32"))]
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if let Some(p) = state.write().play.as_mut() {
                p.tick(100);
            }
        }
    });

    let mut gen_mode    = use_signal(|| GenMode::Og);
    let mut show_result = use_signal(|| false);

    let has_game      = state.read().play.is_some();
    let _complete     = state.read().play.as_ref().map(|p| p.is_complete()).unwrap_or(false);
    let elapsed_ms    = state.read().play.as_ref().map(|p| p.elapsed_ms()).unwrap_or(0);
    let mistakes      = state.read().play.as_ref().map(|p| p.mistakes()).unwrap_or(0);
    let guesses       = state.read().play.as_ref().map(|p| p.guesses()).unwrap_or(0);
    let diff_total    = state.read().play.as_ref().map(|p| p.difficulty_total()).unwrap_or(0);
    let diff_done     = state.read().play.as_ref().map(|p| p.difficulty_done()).unwrap_or(0);
    let total_empties = state.read().play.as_ref().map(|p| p.total_empties()).unwrap_or(0);
    let cells_filled  = state.read().play.as_ref().map(|p| p.cells_filled()).unwrap_or(0);
    let board_given: Option<(BitBoard, BitBoard)> = state.read().play.as_ref()
        .map(|p| (p.board().clone(), p.given().clone()));

    let total_secs = elapsed_ms / 1000;
    let time_str   = format!("{:02}:{:02}.{:03}", total_secs / 60, total_secs % 60, elapsed_ms % 1000);

    let cps_str = if elapsed_ms > 0 {
        format!("{:.2}", cells_filled as f64 / (elapsed_ms as f64 / 1000.0))
    } else {
        "—".to_string()
    };
    let pct_str = if total_empties > 0 {
        format!("{:.0}%", 100.0 * cells_filled as f64 / total_empties as f64)
    } else { "—".to_string() };
    let diff_str = if diff_total > 0 {
        format!("{} / {}", diff_done, diff_total)
    } else { "—".to_string() };

    let _plural = if mistakes == 1 { "" } else { "s" };
    let is_og   = *gen_mode.read() == GenMode::Og;
    let mut n_buf = use_signal(|| "6".to_string());

    rsx! {
        div {
            style: "display: flex; height: 100%; overflow: hidden;",

            // ── LEFT PANEL ─────────────────────────────────────────────────
            div {
                style: "
                    width: 210px; min-width: 210px;
                    background: {PANEL_BG};
                    border-right: 1px solid {BORDER};
                    padding: 14px 12px;
                    display: flex; flex-direction: column; gap: 18px;
                    overflow-y: auto;
                ",

                PlaySection { title: "New Game",
                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            label { style: "font-size: 12px; color: {MUTED}; width: 34px; flex-shrink: 0;", "Size" }
                            input {
                                r#type: "number", min: "2", max: "16", step: "2",
                                value: "{n_buf}",
                                style: "min-width: 0; flex: 1; padding: 6px 8px; background: {INPUT_BG}; border: 1px solid {BTN_BORD}; border-radius: 6px; color: #e0e0e0;",
                                oninput: move |e| *n_buf.write() = e.value(),
                            }
                        }
                        div { style: "display: flex; gap: 0;",
                            PlaySegBtn { label: "Original", active: is_og,  pos: "left",
                                onclick: move |_| *gen_mode.write() = GenMode::Og }
                            PlaySegBtn { label: "Extended", active: !is_og, pos: "right",
                                onclick: move |_| *gen_mode.write() = GenMode::Extended }
                        }
                        button {
                            style: "padding: 8px 0; background: #4a4a4a; color: #f0f0f0; border: none; border-radius: 6px; font-weight: 600; width: 100%;",
                            onclick: move |_| {
                                // Record any completed game that hasn't been dismissed yet.
                                state.write().record_completion_if_done();
                                let n    = n_buf.read().trim().parse::<usize>().unwrap_or(6);
                                let mode = *gen_mode.read();
                                apply(&mut state.write().play, PlayAction::NewGame { n, seed: None, mode });
                                *show_result.write() = false;
                            },
                            "New Game"
                        }
                    }
                }

                if has_game {
                    PlaySection { title: "Stats",
                        div { style: "display: flex; flex-direction: column; gap: 8px;",
                            PlayStatRow { label: "Time",       value: time_str.clone() }
                            PlayStatRow { label: "Cells/s",    value: cps_str }
                            PlayStatRow { label: "Filled",     value: pct_str }
                            PlayStatRow { label: "Difficulty", value: diff_str }
                            PlayStatRow { label: "Mistakes",   value: format!("{mistakes}") }
                            PlayStatRow { label: "Guesses",    value: format!("{guesses}") }
                        }
                    }

                    div { style: "display: flex; flex-direction: column; gap: 6px;",
                        button {
                            style: "padding: 7px 0; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px; width: 100%;",
                            onclick: move |_| apply(&mut state.write().play, PlayAction::Undo),
                            "Undo"
                        }
                        ExportSeed { state }
                    }
                }
            }

            // ── BOARD ──────────────────────────────────────────────────────
            div {
                style: "flex: 1; display: flex; align-items: center; justify-content: center; overflow: auto; background: #1e1e1e; position: relative;",

                if !has_game {
                    span { style: "color: {MUTED}; font-size: 15px;", "Pick a size and hit New Game" }
                }

                if let Some((board_snap, given_snap)) = board_given {
                    div { style: "position: relative;",
                        Board {
                            board: board_snap,
                            locked: Some(given_snap),
                            overlay: None,
                            overlay_step: 0,
                            show_signatures: false,
                            on_cell_click: move |(r, c)| {
                                if state.read().play.as_ref().map(|p| p.is_complete()).unwrap_or(false) { return; }
                                let cur = state.read().play.as_ref()
                                    .map(|p| p.board().get((r, c)))
                                    .unwrap_or(Cell::Nothing);
                                // LMB: Nothing→Red→Blue→Nothing (unchanged)
                                let next = match cur {
                                    Cell::Nothing => Cell::Red,
                                    Cell::Red     => Cell::Blue,
                                    Cell::Blue    => Cell::Nothing,
                                };
                                apply(&mut state.write().play, PlayAction::SetCell(r, c, next));
                                if state.read().play.as_ref().map(|p| p.is_complete()).unwrap_or(false) {
                                    *show_result.write() = true;
                                }
                            },
                            on_right_click: Some(EventHandler::new(move |(r, c)| {
                                if state.read().play.as_ref().map(|p| p.is_complete()).unwrap_or(false) { return; }
                                let cur = state.read().play.as_ref()
                                    .map(|p| p.board().get((r, c)))
                                    .unwrap_or(Cell::Nothing);
                                // RMB: Nothing→Blue→Red→Nothing (reversed cycle)
                                let next = match cur {
                                    Cell::Nothing => Cell::Blue,
                                    Cell::Blue    => Cell::Red,
                                    Cell::Red     => Cell::Nothing,
                                };
                                apply(&mut state.write().play, PlayAction::SetCell(r, c, next));
                                if state.read().play.as_ref().map(|p| p.is_complete()).unwrap_or(false) {
                                    *show_result.write() = true;
                                }
                            })),
                        }

                        // Result popup — shown after completion, board stays visible beneath.
                        if *show_result.read() {
                            div {
                                style: "
                                    position: absolute; inset: 0;
                                    display: flex; align-items: center; justify-content: center;
                                    background: rgba(0,0,0,0.55); border-radius: 12px;
                                ",
                                div {
                                    style: "
                                        background: #1a241a; border: 1px solid #3a6a3a;
                                        border-radius: 14px; padding: 28px 36px;
                                        text-align: center; display: flex; flex-direction: column; gap: 10px;
                                        min-width: 220px;
                                    ",
                                    p { style: "font-size: 24px; font-weight: 700; color: #5dba6a; margin: 0;", "Solved!" }
                                    p { style: "font-size: 15px; color: #8fc898; margin: 0; font-variant-numeric: tabular-nums;",
                                        "{time_str}"
                                    }
                                    div { style: "display: flex; justify-content: center; gap: 20px; font-size: 13px; color: #a0bfa0;",
                                        span { "✗ {mistakes}" }
                                        span { "? {guesses}" }
                                        if diff_total > 0 {
                                            span { "Δ {diff_done}/{diff_total}" }
                                        }
                                    }
                                    div { style: "display: flex; gap: 8px; margin-top: 6px;",
                                        button {
                                            style: "flex: 1; padding: 8px 6px; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 8px; font-size: 13px; white-space: nowrap;",
                                            onclick: move |_| { *show_result.write() = false; },
                                            "View Board"
                                        }
                                        button {
                                            style: "flex: 1; padding: 8px 6px; background: #3a6a3a; color: #e0f0e0; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; white-space: nowrap;",
                                            onclick: move |_| {
                                                state.write().record_completion_if_done();
                                                let n    = n_buf.read().trim().parse::<usize>().unwrap_or(6);
                                                let mode = *gen_mode.read();
                                                apply(&mut state.write().play, PlayAction::NewGame { n, seed: None, mode });
                                                *show_result.write() = false;
                                            },
                                            "Play Again"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── RIGHT PANEL — Game history ─────────────────────────────────
            div {
                style: "
                    width: 220px; min-width: 220px;
                    background: {PANEL_BG};
                    border-left: 1px solid {BORDER};
                    display: flex; flex-direction: column;
                    overflow: hidden;
                ",
                p {
                    style: "margin: 0; padding: 12px 14px 8px; font-size: 11px; font-weight: 600; letter-spacing: .07em; text-transform: uppercase; color: {MUTED}; border-bottom: 1px solid {BORDER};",
                    "History"
                }
                div {
                    style: "flex: 1; overflow-y: auto; padding: 8px 10px; display: flex; flex-direction: column; gap: 8px;",
                    if state.read().play_history.is_empty() {
                        p { style: "font-size: 13px; color: {MUTED}; text-align: center; margin-top: 24px;",
                            "No games yet"
                        }
                    }
                    // Newest first.
                    for (idx, rec) in state.read().play_history.iter().rev().cloned().enumerate() {
                        {
                            let secs = rec.elapsed_ms / 1000;
                            let t    = format!("{:02}:{:02}.{:03}", secs / 60, secs % 60, rec.elapsed_ms % 1000);
                            let mode_label = if rec.mode == GenMode::Og { "OG" } else { "Ext" };
                            let diff_label = if rec.difficulty_total > 0 {
                                format!("{}/{}", rec.difficulty_done, rec.difficulty_total)
                            } else { "—".to_string() };
                            rsx! {
                                div {
                                    key: "{idx}",
                                    style: "
                                        background: #202020; border: 1px solid {BORDER};
                                        border-radius: 8px; padding: 9px 11px;
                                        display: flex; flex-direction: column; gap: 5px;
                                    ",
                                    div { style: "display: flex; justify-content: space-between; align-items: baseline;",
                                        span { style: "font-size: 13px; font-weight: 600; color: #5dba6a;", "{t}" }
                                        span { style: "font-size: 11px; color: {MUTED};", "{rec.n}×{rec.n} {mode_label}" }
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 12px; color: #a0a0a0;",
                                        span { "✗ {rec.mistakes}" }
                                        span { "? {rec.guesses}" }
                                        span { "Δ {diff_label}" }
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

// ── Export seed ───────────────────────────────────────────────────────────────

#[component]
fn ExportSeed(state: Signal<AppState>) -> Element {
    let mut seed_text = use_signal(String::new);
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 4px;",
            button {
                style: "padding: 7px 0; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px; width: 100%; font-size: 12px;",
                onclick: move |_| {
                    *seed_text.write() = state.read().play.as_ref()
                        .map(|p| p.given_seed())
                        .unwrap_or_default();
                },
                "Export Seed"
            }
            if !seed_text.read().is_empty() {
                textarea {
                    style: "width: 100%; height: 60px; background: #1a1a1a; color: #e0e0e0; border: 1px solid {BTN_BORD}; border-radius: 6px; padding: 5px; resize: vertical; font-size: 11px; font-family: monospace;",
                    readonly: true,
                    value: "{seed_text}",
                }
            }
        }
    }
}

// ── Section wrapper ───────────────────────────────────────────────────────────

#[component]
fn PlaySection(title: &'static str, children: Element) -> Element {
    rsx! {
        div {
            p { style: "margin: 0 0 8px; font-size: 11px; font-weight: 600; letter-spacing: .07em; text-transform: uppercase; color: {MUTED};", "{title}" }
            {children}
        }
    }
}

// ── Segmented control button ──────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PlaySegBtnProps {
    label:   &'static str,
    active:  bool,
    pos:     &'static str,
    onclick: EventHandler<()>,
}

#[component]
fn PlaySegBtn(props: PlaySegBtnProps) -> Element {
    let radius = match props.pos {
        "left"  => "6px 0 0 6px",
        "right" => "0 6px 6px 0",
        _       => "0",
    };
    let (bg, fg, fw) = if props.active { ("#505050", "#f0f0f0", "600") } else { (BTN_BG, "#909090", "400") };
    let border = if props.pos == "right" {
        format!("border: 1px solid {BTN_BORD}; border-left: none;")
    } else {
        format!("border: 1px solid {BTN_BORD};")
    };
    let style = format!(
        "flex: 1; padding: 5px 0; background: {bg}; color: {fg}; {border} \
         border-radius: {radius}; font-weight: {fw}; text-align: center; font-size: 12px;"
    );
    rsx! {
        button { style: "{style}", onclick: move |_| props.onclick.call(()), "{props.label}" }
    }
}

// ── Stat row ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PlayStatRowProps { label: &'static str, value: String }

#[component]
fn PlayStatRow(props: PlayStatRowProps) -> Element {
    rsx! {
        div { style: "display: flex; justify-content: space-between; font-size: 13px;",
            span { style: "color: {MUTED};", "{props.label}" }
            span { style: "color: #d0d0d0; font-variant-numeric: tabular-nums;", "{props.value}" }
        }
    }
}
