use std::fmt::{Debug, Formatter};

#[derive(Default)]
pub struct Position {
    pub col: usize,
    pub line: usize,
}

impl Position {
    pub fn m(col: usize, line: usize) -> Self{
        Self {
            col,
            line,
        }
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("line", &self.line)
            .field("col", &self.col)
            .finish()
    }
}