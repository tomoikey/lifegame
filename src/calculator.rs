use crate::cell::{Cell, OwnedCells};
use rand::Rng;
use tokio::sync::mpsc::Sender;

pub struct Calculator {
    width: u16,
    height: u16,
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

        Self {
            width,
            height,
            cells,
            sender,
        }
    }

    fn next(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..self.height as usize {
            for x in 0..self.width as usize {
                let mut living_neighbors = 0;
                for dy in 0..=2 {
                    for dx in 0..=2 {
                        if dy == 1 && dx == 1 {
                            continue;
                        }
                        let y = (y + dy + self.height as usize - 1) % self.height as usize;
                        let x = (x + dx + self.width as usize - 1) % self.width as usize;
                        if self.cells[y][x] == Cell::Living {
                            living_neighbors += 1;
                        }
                    }
                }
                next[y][x] = match (self.cells[y][x], living_neighbors) {
                    (Cell::Empty, 3) => Cell::Living,
                    (Cell::Living, 2..=3) => Cell::Living,
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
