use crate::cell::{Cell, OwnedCells};
use crossterm::terminal;
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
                if rng.random_bool(0.2) {
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

    fn set_aspects(&mut self, width: u16, height: u16) {
        self.set_width(width);
        self.set_height(height);
    }

    fn set_width(&mut self, width: u16) {
        if self.width != width {
            self.width = width;
            for row in &mut self.cells {
                row.resize(width as usize, Cell::Empty);
            }
        }
    }

    fn set_height(&mut self, height: u16) {
        if self.height != height {
            self.height = height;
            self.cells
                .resize(height as usize, vec![Cell::Empty; self.width as usize]);
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
            let (width, height) = terminal::size().expect("terminal::size failed");
            self.set_aspects(width, height);
            self.next();
            self.sender
                .send(self.cells.clone())
                .await
                .expect("channel closed");
        }
    }
}
