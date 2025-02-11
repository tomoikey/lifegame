use crate::cell::{Cell, OwnedCells};
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, PrintStyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType};
use std::io::{stdout, Write};
use tokio::sync::mpsc::Receiver;

pub struct Drawer {
    /// From Holder
    receiver: Receiver<OwnedCells>,
}

impl Drawer {
    pub fn new(receiver: Receiver<OwnedCells>) -> Self {
        Self { receiver }
    }

    pub fn draw(cells: OwnedCells) {
        let mut stdout = stdout();
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
        for row in cells {
            for cell in row {
                match cell {
                    Cell::Empty => {
                        queue!(stdout, PrintStyledContent(" ".with(Color::Black))).unwrap()
                    }
                    Cell::Living => {
                        queue!(stdout, PrintStyledContent("o".with(Color::White))).unwrap()
                    }
                }
            }
        }
        stdout.flush().unwrap();
    }

    pub async fn run(mut self) {
        loop {
            let cells = self.receiver.recv().await.expect("channel closed");
            Self::draw(cells);
        }
    }
}
