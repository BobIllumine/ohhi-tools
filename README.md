# ohhi-tools

A small toolkit for researching Takuzu (also known as Binairo), built in Rust. Designed specifically to aid people in [0h h1](https://0hh1.com/) speedrunning community research possible optimizations in the logic.


## The puzzle

0h h1 is played on an N×N grid (N always even). A completed board is valid when all three rules hold:

1. **Rule of 3** — no three adjacent cells in the same row or column share the same color.
2. **Equity** — every row and column contains exactly N/2 red cells and N/2 blue cells.
3. **No duplicates** — all rows are distinct, and all columns are distinct.

Rules 1 and 2 are local (line-by-line). Rule 3 is global and is the subject of most of the research here.

---
## The goals

In general, little is known about the puzzle mathematically, most of the things we currently use for speedrunning are trivial and we can improve on this significantly.
Some of the goals this tool will be trying to achieve:

1. [ ] Find a new reliable and easy-to-find mechanic for solving the puzzle. Maybe some non-evident patterns or a hidden rule?
2. [ ] Get a good grasp on the puzzle's solvability. Example: how many valid solutions are there for a given board? Is it possible to solve the board only using a restricted set of techniques?
3. [ ] Bring out some numbers, any info is always useful. How many boards on average require guessing? How many valid boards are there for a given board size?

This list will probably grow down the line. It would be easier to check out the code of the game to figure out certain mechanics (RNG in particular), but I plan to do it a bit later.

---

## Workspace layout

```
ohhi-tools/
├── src/
│   ├── ohhi-core/      # Board types, validator, bitwise analytics
│   ├── ohhi-solver/    # Backtracker, deduction engine, carver
│   ├── ohhi-gui/       # egui desktop app
│   └── bruteforcer/    # Parallel brute-force enumerator (research oracle)
└── src/main.rs         # Scratch binary (timing runs, ad-hoc experiments)
```

The crates form a one-way dependency chain:

```
ohhi-core  ←  ohhi-solver  ←  ohhi-gui
                              bruteforcer
```

---

## `ohhi-core`

**What it is:** The shared data layer. Everything else builds on this.

**Key types:**

| Type | Where | What it does |
|---|---|---|
| `Cell` | `board` | Enum: `Red`, `Blue`, `Nothing`. `flip()` swaps colors; `next()` cycles for UI clicks. |
| `BitBoard` | `bit_board` | Primary board representation. Dual bitpacked storage: four `Vec<u16>` masks (red/blue × row/col) give O(1) access to any row or column as a bitmask. |
| `NumTransforms` | `stats` | Trait on `BitBoard`. `signature_x/y`, `count_x/y`, `has_consecutive_x/y`, `is_complete_x/y`. All line-level analytics live here. |
| `Validator` / `Filter` / `Violation` | `validator` | `board.validate(&filter)` checks all three rules. `Filter` lets you disable any rule selectively — useful for counting boards that don't need the duplication rule at all. |

**Bit layout:** LSB-first. Bit `c` of a row mask = column `c`. Bit `r` of a column mask = row `r`. Row and column masks are kept in sync by `BitBoard::set`, which always clears both axes before writing.

**Building:**
```sh
cargo build -p ohhi-core
cargo test -p ohhi-core
```

---

## `ohhi-solver`

**What it is:** Three algorithms that operate on `BitBoard`.

### Backtracker (`backtrack`)

Depth-first search over empty cells. `SolverState::place` prunes illegal branches immediately using the three rules, so the tree is small.

```rust
use ohhi_core::bit_board::BitBoard;
use ohhi_solver::backtrack::calculate;

// How many valid completions does this partial board have?
let n = calculate(&board, usize::MAX);

// Uniqueness test — stop at 2 (much faster than counting all):
let unique = calculate(&board, 2) == 1;
```

`backtrack_one` returns one valid completion, or `None` if none exists.

### Deduction engine (`deduction`)

Applies four bitwise techniques in a fixpoint loop, recording each forced cell and the rule that forced it. Useful for attributing solves to specific rules.

```rust
use ohhi_solver::deduction::{deduce, deduce_with, TechniqueSet, Technique};

// Run all four techniques:
let trace = deduce(&board);

// Run without the duplication rule (simulates "gambling" seeds):
let trace = deduce_with(&board, TechniqueSet::ALL.without(Technique::TwinCompletion));

for step in trace.get_steps() {
    println!("{:?} at ({}, {}) forced by {:?}", step.cell, step.at.0, step.at.1, step.technique);
}

if trace.stalled {
    println!("engine hit a contradiction");
} else if trace.get_steps().is_empty() {
    println!("no technique fired — board requires guessing");
}
```

**Techniques** (applied in priority order, rows before columns):

| Technique | Pattern | Rule |
|---|---|---|
| `PairExtension` | `XX_` or `_XX` | Rule of 3 |
| `GapFill` | `X_X` | Rule of 3 |
| `Saturation` | line has N/2 of one color | Equity |
| `TwinCompletion` | line has 2 empties, one completion would duplicate an existing line | No duplicates |

### Carver (`carver`)

Given a complete valid board, removes cells one at a time (random order) while the solution stays unique. Returns the minimal seed — every clue that remains is necessary.

```rust
use ohhi_solver::carver::carve;

let seed = carve(&complete_board).unwrap();
// seed has exactly one valid completion
```

Repeated calls on the same board typically produce different seeds of similar clue-count.

**Building:**
```sh
cargo build -p ohhi-solver
cargo test -p ohhi-solver
```

---

## `ohhi-gui`

**What it is:** A desktop app (egui / eframe) for interactively exploring boards and running the solver tools.

**Running:**
```sh
cargo run --release -p ohhi-gui
```

**Layout:**

```
┌─────────────────┬──────────────────────────┬──────────────────────┐
│  Left toolbar   │       Board canvas        │   Right panel        │
│                 │                           │                       │
│  Undo / Redo    │  Click cell → cycle color │  Validate             │
│  Clear          │                           │  Show signatures      │
│  Resize         │  Overlay: forced cells    │  Filter rules         │
│                 │  shown dimmed with        │                       │
│  ─────────────  │  technique-colored border │  ─────────────────── │
│                 │  and letter label:        │  Deduction techniques │
│  Load Seed      │    P  G  S  T             │  Deduce               │
│                 │                           │  Apply trace          │
│                 │  Signatures: row sigs     │  Clear overlay        │
│                 │  left, col sigs below     │  Step slider          │
└─────────────────┴──────────────────────────┴──────────────────────┘
```

**Seed format** — paste into the Load Seed dialog:
```
R . B .
. R . B
B . R .
. B . R
```
Each cell is `R`, `B`, or `.`, separated by spaces. Rows separated by newlines. Maximum board size is 16×16.

**Deduction overlay:** click Deduce to run the engine on the current board. Forced cells appear dimmed with a thick colored border and a letter showing which technique fired. Use the slider to scrub through steps. "Apply trace" commits the final board state and clears the overlay.

**Architecture:** state lives in `GuiState` in `state.rs`. All mutations go through `apply(state, action)`. Render functions in `render.rs` read state and fire `Action` values — nothing mutates state directly from the render path.

---

## `bruteforcer`

**What it is:** An independent oracle for small boards. Enumerates every possible coloring of an N×N grid and validates each one, then counts how many are valid. Used to cross-check the backtracker's solution counts.

**Running:**
```sh
# Edit the `n` constant in src/bruteforcer/src/main.rs, then:
cargo run --release -p bruteforcer
```

Practical limits: 4×4 runs in milliseconds, 6×6 in seconds. 8×8 would take days — use the backtracker for anything larger.

Uses `ThreadPool<I, O>` from `bruteforcer::thread_pool` — a generic fan-out/fan-in worker pool that distributes work items across N threads and collects `(input, output)` pairs.

---

## Building and testing

```sh
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Build GUI in release mode (much faster at runtime)
cargo run --release -p ohhi-gui

# Generate and open API docs
cargo doc --workspace --no-deps --open
```

Linux requires system GUI libraries for eframe:
```sh
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

---
