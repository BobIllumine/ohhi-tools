use dioxus::prelude::*;
use ohhi_app::AppState;
use ohhi_app::practice::{Attempt, PracticeSession, Target};
use ohhi_core::board::Cell;

use super::board::Board;

const PANEL_BG: &str = "#181818";
const BORDER:   &str = "#2e2e2e";
const BTN_BG:   &str = "#2a2a2a";
const BTN_FG:   &str = "#e0e0e0";
const BTN_BORD: &str = "#404040";
const MUTED:    &str = "#909090";

const TARGETS: [(Target, &str); 4] = [
    (Target::Saturation, "Saturation"),
    (Target::TwinCompletion, "Twin completion"),
    (Target::PairExtension, "Pair extension"),
    (Target::GapFill, "Gap fill"),
];

#[component]
pub fn PracticeView(mut state: Signal<AppState>) -> Element {
    // Background timer: tick every 100 ms (desktop only).
    #[cfg(not(target_arch = "wasm32"))]
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if let Some(p) = state.write().practice.as_mut() {
                p.tick(100);
            }
        }
    });

    let has_drill = state.read().practice.is_some();
    if !has_drill {
        return rsx! { SetupView { state } };
    }
    rsx! { DrillView { state } }
}

// ── Setup ───────────────────────────────────────────────────────────────────

#[component]
fn SetupView(mut state: Signal<AppState>) -> Element {
    let mut target = use_signal(|| Target::Saturation);
    let mut reps   = use_signal(|| 10usize);
    let mut size   = use_signal(|| 6usize);
    let mut error  = use_signal(|| false);

    rsx! {
        div {
            style: "height: 100%; display: flex; align-items: center; justify-content: center; background: #1e1e1e;",
            div {
                style: "width: 360px; background: {PANEL_BG}; border: 1px solid {BORDER}; border-radius: 12px; padding: 24px; display: flex; flex-direction: column; gap: 18px;",
                h2 { style: "margin: 0; font-size: 18px;", "Practice drill" }
                p { style: "margin: 0; font-size: 13px; color: {MUTED};",
                    "Spot every cell the chosen technique forces — and only those. Guessing a cell the technique doesn't justify is a miss."
                }

                div { style: "display: flex; flex-direction: column; gap: 8px;",
                    span { style: "font-size: 12px; color: {MUTED};", "Technique" }
                    for (t, label) in TARGETS {
                        button {
                            style: {
                                let active = *target.read() == t;
                                let (bg, fw) = if active { ("#505050", "600") } else { (BTN_BG, "400") };
                                format!("padding: 8px 12px; background: {bg}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 6px; font-weight: {fw}; text-align: left;")
                            },
                            onclick: move |_| *target.write() = t,
                            "{label}"
                        }
                    }
                }

                div { style: "display: flex; gap: 12px;",
                    label { style: "flex: 1; font-size: 12px; color: {MUTED};",
                        "Board size"
                        input {
                            r#type: "number", min: "4", max: "10", step: "2", value: "{size}",
                            style: "width: 100%; margin-top: 4px; padding: 6px; background: #1a1a1a; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                            oninput: move |e| if let Ok(v) = e.value().parse() { *size.write() = v; }
                        }
                    }
                    label { style: "flex: 1; font-size: 12px; color: {MUTED};",
                        "Reps"
                        input {
                            r#type: "number", min: "1", max: "100", value: "{reps}",
                            style: "width: 100%; margin-top: 4px; padding: 6px; background: #1a1a1a; border: 1px solid {BTN_BORD}; border-radius: 6px;",
                            oninput: move |e| if let Ok(v) = e.value().parse() { *reps.write() = v; }
                        }
                    }
                }

                if *error.read() {
                    p { style: "margin: 0; font-size: 12px; color: #e05555;",
                        "Couldn't generate a board for that technique/size — try another."
                    }
                }

                button {
                    style: "padding: 10px; background: #4a4a4a; color: #f0f0f0; border: none; border-radius: 8px; font-weight: 600;",
                    onclick: move |_| {
                        let seed = rand_seed();
                        let s = PracticeSession::new(*target.read(), *size.read(), *reps.read(), seed);
                        match s {
                            Some(sess) => { state.write().practice = Some(sess); *error.write() = false; }
                            None => *error.write() = true,
                        }
                    },
                    "Start"
                }
            }
        }
    }
}

// ── Drill ───────────────────────────────────────────────────────────────────

#[component]
fn DrillView(mut state: Signal<AppState>) -> Element {
    let mut last = use_signal(|| None::<Attempt>);

    let snap = state.read();
    let p = snap.practice.as_ref().unwrap();
    let board = p.board().clone();
    let complete = p.is_complete();
    let (hits, misses, total) = (p.hits(), p.misses(), p.total_reps());
    let rep = p.current_rep();
    let available = p.available();
    let elapsed = p.elapsed_ms();
    let mean = p.mean_time_ms();
    let accuracy = p.accuracy();
    let target_label = target_name(p.target());
    drop(snap);

    let mean_str = mean.map(|m| format!("{:.2}s", m as f64 / 1000.0)).unwrap_or_else(|| "—".into());
    let acc_str  = format!("{:.0}%", accuracy * 100.0);

    rsx! {
        div {
            style: "display: flex; height: 100%; background: #1e1e1e;",

            // Stats sidebar
            div {
                style: "width: 220px; background: {PANEL_BG}; border-right: 1px solid {BORDER}; padding: 16px; display: flex; flex-direction: column; gap: 14px;",
                h3 { style: "margin: 0; font-size: 15px;", "{target_label}" }
                Stat { label: "Rep", value: format!("{rep}/{total}") }
                Stat { label: "Hits", value: hits.to_string() }
                Stat { label: "Misses (guesses)", value: misses.to_string() }
                Stat { label: "Accuracy", value: format!("{:.0}%", accuracy * 100.0) }
                Stat { label: "Mean time", value: mean_str.clone() }
                Stat { label: "This rep", value: format!("{:.1}s", elapsed as f64 / 1000.0) }
                if !complete {
                    Stat { label: "Cells to find", value: available.to_string() }
                }

                if let Some(a) = *last.read() {
                    div {
                        style: {
                            let (bg, fg, txt) = match a {
                                Attempt::Hit => ("#1c2e1c", "#7bd87b", "✓ Hit"),
                                Attempt::Miss => ("#2e1c1c", "#e07b7b", "✗ Guess"),
                                Attempt::Ignored => ("#222", "#888", "—"),
                            };
                            let _ = txt;
                            format!("padding: 8px; border-radius: 6px; background: {bg}; color: {fg}; font-weight: 600; text-align: center;")
                        },
                        {match a {
                            Attempt::Hit => "✓ Hit",
                            Attempt::Miss => "✗ Guess",
                            Attempt::Ignored => "—",
                        }}
                    }
                }

                div { style: "flex: 1;" }
                button {
                    style: "padding: 9px; background: {BTN_BG}; color: {BTN_FG}; border: 1px solid {BTN_BORD}; border-radius: 8px;",
                    onclick: move |_| { state.write().practice = None; },
                    "End drill"
                }
            }

            // Board area
            div {
                style: "flex: 1; display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 16px;",
                if complete {
                    div {
                        style: "background: {PANEL_BG}; border: 1px solid {BORDER}; border-radius: 12px; padding: 28px; text-align: center; display: flex; flex-direction: column; gap: 10px;",
                        h2 { style: "margin: 0;", "Drill complete" }
                        p { style: "margin: 0; color: {MUTED};", "Mean {mean_str} · {acc_str} accuracy" }
                        button {
                            style: "margin-top: 8px; padding: 9px 16px; background: #4a4a4a; color: #f0f0f0; border: none; border-radius: 8px; font-weight: 600;",
                            onclick: move |_| { state.write().practice = None; },
                            "New drill"
                        }
                    }
                } else {
                    p { style: "margin: 0; font-size: 13px; color: {MUTED};",
                        "Left-click = Red · Right-click = Blue"
                    }
                    Board {
                        board: board.clone(),
                        locked: Some(board.clone()),
                        on_cell_click: move |(r, c)| {
                            let a = state.write().practice.as_mut().map(|p| p.attempt((r, c), Cell::Red));
                            if let Some(a) = a { *last.write() = Some(a); }
                            commit_if_done(state);
                        },
                        on_right_click: Some(EventHandler::new(move |(r, c)| {
                            let a = state.write().practice.as_mut().map(|p| p.attempt((r, c), Cell::Blue));
                            if let Some(a) = a { *last.write() = Some(a); }
                            commit_if_done(state);
                        })),
                    }
                }
            }
        }
    }
}

#[component]
fn Stat(label: &'static str, value: String) -> Element {
    rsx! {
        div { style: "display: flex; justify-content: space-between; font-size: 13px;",
            span { style: "color: {MUTED};", "{label}" }
            span { style: "font-weight: 600; font-variant-numeric: tabular-nums;", "{value}" }
        }
    }
}

fn target_name(t: Target) -> &'static str {
    match t {
        Target::Saturation => "Saturation",
        Target::TwinCompletion => "Twin completion",
        Target::PairExtension => "Pair extension",
        Target::GapFill => "Gap fill",
        Target::Counting => "Counting",
    }
}

fn rand_seed() -> u64 {
    use std::hash::{BuildHasher, RandomState};
    RandomState::new().hash_one("practice")
}

/// If the drill just finished, log it to history and persist to disk (once).
fn commit_if_done(mut state: Signal<AppState>) {
    let recorded = state.write().record_drill_if_done(now_ms()).is_some();
    if recorded {
        let history = state.read().practice_history.clone();
        crate::storage::save(&history);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> u64 { 0 }
