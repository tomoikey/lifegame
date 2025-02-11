use crate::cell::{Cell, OwnedCells};
use rand::Rng;
use tokio::sync::mpsc::Sender;

pub struct Calculator {
    cells: OwnedCells,
    /// To Holder
    sender: Sender<OwnedCells>,
}

impl Calculator {
    pub fn new(width: u16, height: u16, sender: Sender<OwnedCells>) -> Self {
        let mut cells = vec![vec![Cell::Empty; width as usize]; height as usize];

        let mut rng = rand::rng();
        for row in &mut cells {
            for cell in row {
                if rng.random_bool(0.1) {
                    *cell = Cell::Living;
                }
            }
        }

        Self { cells, sender }
    }

    fn next(&mut self) {
        let mut next = self.cells.clone();
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let mut living = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dy == 0 && dx == 0 {
                            continue;
                        }
                        let y = y as i32 + dy;
                        let x = x as i32 + dx;
                        if y < 0 || self.cells.len() as i32 <= y {
                            continue;
                        }
                        if x < 0 || self.cells[y as usize].len() as i32 <= x {
                            continue;
                        }
                        if self.cells[y as usize][x as usize] == Cell::Living {
                            living += 1;
                        }
                    }
                }
                next[y][x] = match (cell, living) {
                    (Cell::Empty, 3) => Cell::Living,
                    (Cell::Living, 2) | (Cell::Living, 3) => Cell::Living,
                    _ => Cell::Empty,
                };
            }
        }
        self.cells = next;
    }

    pub async fn run(mut self) {
        loop {
            self.next();
            self.sender
                .send(self.cells.clone())
                .await
                .expect("channel closed");
        }
    }
}
