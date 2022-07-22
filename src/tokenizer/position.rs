use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    fn make(col: usize, row: usize) -> Self{
        Self {
            col,
            row,
        }
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("col", &self.col)
            .field("row", &self.row)
            .finish()
    }
}