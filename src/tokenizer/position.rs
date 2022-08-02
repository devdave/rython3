use std::fmt::{Debug, Formatter};

#[derive(Default, Copy, Clone)]
pub struct Position {
    pub col: usize,
    pub line: usize,
}

impl Position {
    pub fn m(col: usize, line: usize) -> Self{
        Self {
            col:col,
            line:line.saturating_add(1),
        }
    }

    pub fn t((col, line): (usize, usize)) -> Self {
        Self::m(col, line)
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