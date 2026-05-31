use ohhi_core::board::Cell;
use ohhi_generator::practice::{target_forced, targeted_board, Target};
use ohhi_solver::structs::{Technique, TechniqueSet};
use ohhi_solver::v1::deduction::forced_once;
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn technique_of(t: Target) -> Technique {
    match t {
        Target::PairExtension => Technique::PairExtension,
        Target::GapFill => Technique::GapFill,
        Target::Saturation => Technique::Saturation,
        Target::TwinCompletion => Technique::TwinCompletion,
        Target::Counting => unreachable!(),
    }
}

const TARGETS: [Target; 4] = [
    Target::PairExtension,
    Target::GapFill,
    Target::Saturation,
    Target::TwinCompletion,
];

#[test]
fn each_target_generates_a_board() {
    let mut rng = SmallRng::seed_from_u64(7);
    for target in TARGETS {
        let board = targeted_board(target, 6, &mut rng, 20_000)
            .unwrap_or_else(|| panic!("{target:?} produced no board in budget"));

        // (b) the target forces at least one cell...
        let forced = target_forced(&board.given, target);
        assert!(!forced.is_empty(), "{target:?}: target forces nothing");

        // ...and NO other technique forces anything (strict: only target progresses).
        let others = TechniqueSet::ALL.without(technique_of(target));
        assert!(
            forced_once(&board.given, others).is_empty(),
            "{target:?}: another technique can also make progress"
        );

        // ...and every forced cell agrees with the solution (techniques are sound).
        for ((r, c), color) in &forced {
            assert_eq!(
                board.solution.get((*r, *c)),
                *color,
                "{target:?}: forced ({r},{c}) disagrees with solution"
            );
            // forced cells are genuinely empty in the given board
            assert_eq!(board.given.get((*r, *c)), Cell::Nothing);
        }
    }
}

#[test]
fn twin_boards_can_be_partially_filled() {
    // The user's requirement: twin-completion drills shouldn't all be near-complete.
    let mut rng = SmallRng::seed_from_u64(99);
    let board = targeted_board(Target::TwinCompletion, 8, &mut rng, 50_000)
        .expect("twin board");
    let total = board.given.width() * board.given.height();
    let filled = (0..board.given.height())
        .flat_map(|r| (0..board.given.width()).map(move |c| (r, c)))
        .filter(|&(r, c)| board.given.get((r, c)) != Cell::Nothing)
        .count();
    // Sanity: it's a real partial board, not fully given and not empty.
    assert!(filled > 0 && filled < total, "filled={filled}/{total}");
}

#[test]
fn twin_exposes_both_cells_of_the_pair() {
    // A twin pins both empties of the line — one Blue, one Red — and Practice
    // must accept either, not just the Blue one.
    let mut rng = SmallRng::seed_from_u64(3);
    let board = targeted_board(Target::TwinCompletion, 8, &mut rng, 50_000).expect("twin");
    let forced = target_forced(&board.given, Target::TwinCompletion);
    assert!(
        forced.iter().any(|&(_, c)| c == Cell::Blue) && forced.iter().any(|&(_, c)| c == Cell::Red),
        "twin must force both a Blue and a Red cell, got {forced:?}"
    );
}

#[test]
fn counting_target_is_unsupported_for_now() {
    let mut rng = SmallRng::seed_from_u64(1);
    assert!(targeted_board(Target::Counting, 6, &mut rng, 100).is_none());
}
