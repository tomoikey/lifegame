use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode};
use crossterm::style::{Color, PrintStyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, event, terminal};
use crossterm::{execute, queue};
use rand::Rng;
use std::io::{stdout, Result, Write};
use std::process::exit;
use std::thread;

fn exit_on_q_input() -> Result<()> {
    terminal::enable_raw_mode()?;
    loop {
        let event = event::read()?;
        if let Event::Key(key_event) = event {
            if let KeyCode::Char('q') = key_event.code {
                break;
            }
        }
    }
    execute!(stdout(), cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    exit(0);
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        EnterAlternateScreen,
        Clear(ClearType::All)
    )?;

    thread::spawn(|| exit_on_q_input().expect("exit_on_q_input failed"));

    let (width, height) = terminal::size()?;
    let mut screen = Screen::new(width, height);
    loop {
        let (width, height) = terminal::size()?;
        screen.set_aspects(width, height);
        screen.draw();
        screen.next();
    }
}

struct Screen {
    width: u16,
    height: u16,
    cells: Vec<Vec<Cell>>,
}

impl Screen {
    fn new(width: u16, height: u16) -> Self {
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

    fn draw(&self) {
        let mut stdout = stdout();
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();
        for row in &self.cells {
            for cell in row {
                match cell {
                    Cell::Empty => {
                        queue!(stdout, PrintStyledContent(" ".with(Color::Black))).unwrap()
                    }
                    Cell::Living => {
                        queue!(stdout, PrintStyledContent("■".with(Color::White))).unwrap()
                    }
                }
            }
        }
        stdout.flush().unwrap();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Living,
}
