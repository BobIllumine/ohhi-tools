use rand;
use rand::{Rng, RngExt};
use crate::board::{Cell, Line};

pub trait _Randomizer
{
    type Output;
    fn shuffle(&self) -> Self::Output;

    fn rand_new(&self, size: usize) -> Self::Output;
}

impl rand::Fill for Cell {
    fn fill_slice<R: Rng + ?Sized>(this: &mut [Self], rng: &mut R) {
        for mut cell in this.iter_mut() {
            let rand_num: u8 = rng.random_range(0..=2);
            let tmp: Cell;
            match rand_num {
                1 => { tmp = Cell::Red; }
                2 => { tmp = Cell::Blue; }
                _ => { tmp = Cell::Nothing; }
            }
            cell = &mut tmp.clone();
        }
    }
}

impl _Randomizer for Line {
    type Output = Line;
    fn shuffle(&self) -> Line {
        let _rng = rand::rng();
        todo!()
    }

    fn rand_new(&self, size: usize) -> Line {
        let mut rng = rand::rng();
        let mut data = Self::new(size);
        rng.fill(&mut data.as_mut());
        data
    }

}