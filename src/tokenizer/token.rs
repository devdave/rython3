
use std::fmt::{Debug, Display, Formatter};
use crate::tokenizer::position::Position;
use crate::tokenizer::ttype::TType;
// use crate::tokenizer::position::Position;

pub struct Token {
    pub r#type: TType,
    pub start: Position,
    pub end: Position,
    pub text: String,
}

#[allow(non_snake_case)]
impl Token {
    pub(crate) fn Make(ttype: TType, start: Position, end: Position, content: &str) -> Self {
        Self {
            r#type: ttype,
            start: start,
            end: end,
            text: content.to_string(),
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.r#type)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("text", &self.text)
            .finish()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.r#type)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("text", &self.text)
            .finish()
    }

}