use bruteforcer::thread_pool::ThreadPool;
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_core::validator::{Filter, Validator, Violation};
use std::thread;

fn create_and_submit(
    input: &mut Vec<Vec<Cell>>,
    pos: usize,
    thread_pool: &ThreadPool<(BitBoard, Filter), Result<(), Violation>>,
    filter: Filter
)
{
    if pos == input.len() * input[0].len() {
        thread_pool.submit((BitBoard::from(&*input), filter)).unwrap();
        return;
    }
    let r = pos / input[0].len();
    let c = pos % input[0].len();
    for &cell in &[Cell::Red, Cell::Blue] {
        input[r][c] = cell;
        create_and_submit(input, pos + 1, thread_pool, filter);
    }
}

fn main() {
    let num_workers = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let func = |(board, filter): &(BitBoard, Filter)| board.validate(filter);
    let tp = ThreadPool::new(num_workers, func);
    let n = 4;
    let mut input = vec![vec![Cell::Nothing; n]; n];
    create_and_submit(&mut input, 0, &tp, Filter {
        rule_of_3: true,
        rule_of_equity: true,
        rule_of_duplication: true,
        incomplete: true,
    });
    let result = tp.collect();
    println!("num valid boards ({n}x{n}): {}", result.iter().filter(|(_, verdict)| match verdict {
        Ok(_) => true,
        _ => false
    }).count());
}
