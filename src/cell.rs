#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Living,
}

pub type OwnedCells = Vec<Vec<Cell>>;
