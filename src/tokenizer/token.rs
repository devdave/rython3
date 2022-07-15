use std::fmt::{Debug, Formatter};
use crate::tokenizer::ttype::TType;

pub struct Token {
    pub r#type: TType,
    pub lineno: usize,
    pub line_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub text: String,
}

#[allow(non_snake_case)]
impl Token {
    pub(crate) fn Make(ttype: TType, lineno: usize, col_start: usize, col_end: usize, content: &str) -> Self {
        Self {
            r#type: ttype,
            lineno: lineno,
            line_end: lineno,
            col_start: col_start,
            col_end: col_end,
            text: content.to_string(),
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.r#type)
            .field("lineno", &self.lineno)
            .field("col_start", &self.col_start)
            .field("text", &self.text)
            .finish()
    }
}
