use dioxus::prelude::*;
use ohhi_app::history::{summaries, trend};
use ohhi_app::practice::Target;
use ohhi_app::AppState;

const PANEL_BG: &str = "#181818";
const BORDER:   &str = "#2e2e2e";
const MUTED:    &str = "#909090";
const ACCENT:   &str = "#5dba6a";
const ACCENT2:  &str = "#5b9bd5";

#[component]
pub fn StatsView(state: Signal<AppState>) -> Element {
    let history = state.read().practice_history.clone();
    let mut sel = use_signal(|| 0usize);

    if history.is_empty() {
        return rsx! {
            div {
                style: "height: 100%; display: flex; align-items: center; justify-content: center; background: #1e1e1e; color: {MUTED};",
                "No drills yet — finish a Practice drill to start building history."
            }
        };
    }

    let sums = summaries(&history);
    let targets: Vec<Target> = sums.iter().map(|s| s.target).collect();
    let sel_i = *sel.read() % targets.len();
    let sel_target = targets[sel_i];
    let tr = trend(&history, sel_target);
    let means: Vec<u64> = tr.iter().map(|d| d.mean_ms()).collect();
    let accs: Vec<f64> = tr.iter().map(|d| d.accuracy()).collect();
    let last = history.last().unwrap().clone();
    let max_avg = sums.iter().map(|s| s.avg_mean_ms).max().unwrap_or(1).max(1);

    rsx! {
        div {
            style: "height: 100%; overflow-y: auto; background: #1e1e1e; padding: 22px;",
            div {
                style: "max-width: 720px; margin: 0 auto; display: flex; flex-direction: column; gap: 22px;",

                h2 { style: "margin: 0; font-size: 18px;", "Practice stats" }
                p { style: "margin: 0; font-size: 13px; color: {MUTED};", "{history.len()} drills recorded." }

                // ── By technique ─────────────────────────────────────────────
                Section { title: "By technique",
                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        for s in sums.iter() {
                            {
                                let bar_w = s.avg_mean_ms as f64 / max_avg as f64 * 100.0;
                                let caption = format!(
                                    "{} drills · best {} · {:.0}% acc",
                                    s.drills, fmt_ms(s.best_mean_ms), s.avg_accuracy * 100.0
                                );
                                let name = target_name(s.target);
                                rsx! {
                                    div { style: "display: flex; align-items: center; gap: 12px; font-size: 13px;",
                                        span { style: "width: 130px; flex-shrink: 0; font-weight: 600;", "{name}" }
                                        div { style: "flex: 1; height: 18px; background: #222; border-radius: 4px; overflow: hidden;",
                                            div { style: "height: 100%; width: {bar_w:.1}%; background: {ACCENT2};" }
                                        }
                                        span { style: "width: 210px; flex-shrink: 0; color: {MUTED}; font-variant-numeric: tabular-nums;", "{caption}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Trend across drills ──────────────────────────────────────
                Section { title: "Trend across drills",
                    div { style: "display: flex; gap: 6px; margin-bottom: 12px; flex-wrap: wrap;",
                        for (i, t) in targets.iter().enumerate() {
                            button {
                                style: {
                                    let active = i == sel_i;
                                    let (bg, fg) = if active { ("#505050", "#f0f0f0") } else { ("#2a2a2a", "#909090") };
                                    format!("padding: 5px 12px; background: {bg}; color: {fg}; border: 1px solid #404040; border-radius: 6px; font-size: 12px;")
                                },
                                onclick: move |_| *sel.write() = i,
                                "{target_name(*t)}"
                            }
                        }
                    }
                    LineChart { means: means.clone(), accs: accs.clone() }
                    p { style: "margin: 8px 0 0; font-size: 12px; color: {MUTED};",
                        "Mean recognition time (green) and accuracy (blue) over each {target_name(sel_target)} drill, oldest → newest."
                    }
                }

                // ── Most recent recognition curve ────────────────────────────
                Section { title: "Last drill — per-rep time",
                    BarChart { times: last.times.clone() }
                    p { style: "margin: 8px 0 0; font-size: 12px; color: {MUTED};",
                        "{target_name(last.target)} · {last.times.len()} reps · mean {fmt_ms(last.mean_ms())} · best {fmt_ms(last.best_ms())}"
                    }
                }
            }
        }
    }
}

// ── Charts (inline SVG) ───────────────────────────────────────────────────────

const W: f64 = 660.0;
const H: f64 = 180.0;
const PAD: f64 = 12.0;

#[component]
fn BarChart(times: Vec<u64>) -> Element {
    let n = times.len().max(1);
    let max = times.iter().copied().max().unwrap_or(1).max(1) as f64;
    let slot = (W - 2.0 * PAD) / n as f64;
    let bw = (slot * 0.7).max(2.0);

    rsx! {
        svg { width: "100%", height: "{H}", view_box: "0 0 {W} {H}", preserve_aspect_ratio: "none",
            for (i, &v) in times.iter().enumerate() {
                {
                    let h = (v as f64 / max) * (H - 2.0 * PAD);
                    let x = PAD + i as f64 * slot + (slot - bw) / 2.0;
                    let y = H - PAD - h;
                    rsx! { rect { x: "{x}", y: "{y}", width: "{bw}", height: "{h}", rx: "2", fill: "{ACCENT}" } }
                }
            }
            line { x1: "{PAD}", y1: "{H - PAD}", x2: "{W - PAD}", y2: "{H - PAD}", stroke: "#444", stroke_width: "1" }
        }
    }
}

#[component]
fn LineChart(means: Vec<u64>, accs: Vec<f64>) -> Element {
    let n = means.len();
    let max = means.iter().copied().max().unwrap_or(1).max(1) as f64;
    let x_of = move |i: usize| if n <= 1 { W / 2.0 } else { PAD + i as f64 / (n - 1) as f64 * (W - 2.0 * PAD) };
    let y_mean = move |v: u64| H - PAD - (v as f64 / max) * (H - 2.0 * PAD);
    let y_acc  = move |a: f64| H - PAD - a * (H - 2.0 * PAD); // accuracy 0..1

    let mean_pts: String = means.iter().enumerate()
        .map(|(i, &v)| format!("{:.1},{:.1}", x_of(i), y_mean(v))).collect::<Vec<_>>().join(" ");
    let acc_pts: String = accs.iter().enumerate()
        .map(|(i, &a)| format!("{:.1},{:.1}", x_of(i), y_acc(a))).collect::<Vec<_>>().join(" ");

    rsx! {
        svg { width: "100%", height: "{H}", view_box: "0 0 {W} {H}", preserve_aspect_ratio: "none",
            line { x1: "{PAD}", y1: "{H - PAD}", x2: "{W - PAD}", y2: "{H - PAD}", stroke: "#444", stroke_width: "1" }
            polyline { points: "{acc_pts}", fill: "none", stroke: "{ACCENT2}", stroke_width: "2", opacity: "0.8" }
            polyline { points: "{mean_pts}", fill: "none", stroke: "{ACCENT}", stroke_width: "2" }
            for (i, &v) in means.iter().enumerate() {
                circle { cx: "{x_of(i)}", cy: "{y_mean(v)}", r: "3", fill: "{ACCENT}" }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[component]
fn Section(title: &'static str, children: Element) -> Element {
    rsx! {
        div { style: "background: {PANEL_BG}; border: 1px solid {BORDER}; border-radius: 12px; padding: 18px;",
            h3 { style: "margin: 0 0 14px; font-size: 14px; color: #d0d0d0;", "{title}" }
            {children}
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

fn fmt_ms(ms: u64) -> String {
    format!("{:.2}s", ms as f64 / 1000.0)
}
