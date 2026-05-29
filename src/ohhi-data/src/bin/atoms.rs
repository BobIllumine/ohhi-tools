//! Pattern catalog and minimal trigger mining.
//!
//! For each board size n, enumerates all forcing partial-line states and emits:
//! - `data/atoms_full_n{N}.csv` — every forcing state
//! - `data/atoms_minimal_n{N}.csv` — minimal canonical triggers (symmetry-reduced)
//!
//! Usage: cargo run --release -p ohhi-data --bin atoms

use ohhi_data::catalog::{catalog, minimal_triggers, RuleClass};
use std::fs;
use std::io::Write;

fn rule_class_str(r: &RuleClass) -> &'static str {
    match r {
        RuleClass::LocalAntiTriple  => "local_anti_triple",
        RuleClass::Saturation       => "saturation",
        RuleClass::CountingAntiTriple => "counting_anti_triple",
    }
}

fn pattern_str(red: u16, blue: u16, n: usize) -> String {
    (0..n).map(|i| {
        if red  & (1 << i) != 0 { 'R' }
        else if blue & (1 << i) != 0 { 'B' }
        else { '.' }
    }).collect()
}

fn main() {
    fs::create_dir_all("../../../../data").expect("could not create data/");

    for n in [4usize, 6, 8, 10] {
        println!("n={n}: cataloging...");

        // Full catalog CSV.
        let full_entries = catalog(n);
        let full_path = format!("data/atoms_full_n{n}.csv");
        let mut f = fs::File::create(&full_path).expect("create full csv");
        writeln!(f, "n,red_mask,blue_mask,pattern,forced_positions,forced_colors,rule_class").unwrap();
        for e in &full_entries {
            let positions: String = e.forced.iter().map(|(p,_)| p.to_string()).collect::<Vec<_>>().join(";");
            let colors: String = e.forced.iter().map(|(_,c)| match c {
                ohhi_core::board::Cell::Red => "R",
                ohhi_core::board::Cell::Blue => "B",
                _ => ".",
            }).collect::<Vec<_>>().join(";");
            writeln!(f, "{n},{},{},{},{},{},{}",
                e.state.red, e.state.blue,
                pattern_str(e.state.red, e.state.blue, n),
                positions, colors,
                rule_class_str(&e.rule_class)
            ).unwrap();
        }
        println!("  full:    {} states → {full_path}", full_entries.len());

        // Minimal triggers CSV.
        let minimal_entries = minimal_triggers(n);
        let minimal_path = format!("data/atoms_minimal_n{n}.csv");
        let mut f = fs::File::create(&minimal_path).expect("create minimal csv");
        writeln!(f, "n,red_mask,blue_mask,pattern,forced_positions,forced_colors,rule_class").unwrap();
        for e in &minimal_entries {
            let positions: String = e.forced.iter().map(|(p,_)| p.to_string()).collect::<Vec<_>>().join(";");
            let colors: String = e.forced.iter().map(|(_,c)| match c {
                ohhi_core::board::Cell::Red => "R",
                ohhi_core::board::Cell::Blue => "B",
                _ => ".",
            }).collect::<Vec<_>>().join(";");
            writeln!(f, "{n},{},{},{},{},{},{}",
                e.state.red, e.state.blue,
                pattern_str(e.state.red, e.state.blue, n),
                positions, colors,
                rule_class_str(&e.rule_class)
            ).unwrap();
        }
        println!("  minimal: {} states → {minimal_path}", minimal_entries.len());
    }

    println!("\nDone.");
}
