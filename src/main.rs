use std::time::Instant;
use ohhi_core::bit_board;
use ohhi_solver::deduction;
use ohhi_solver::backtrack;

fn main() {
    let start = Instant::now();
    let n = 6;
    let mut board = bit_board::BitBoard::new(n, n);
    println!("Board: \n{board}\n");
    println!("Solutions for {n}x{n}: {}", backtrack::calculate(&mut board, usize::MAX));
    println!("Time: {:?}", start.elapsed());
}