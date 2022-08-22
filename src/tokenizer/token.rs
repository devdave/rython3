
use std::fmt::{Debug, Display, Formatter};
use crate::tokenizer::position::Position;
use crate::tokenizer::ttype::TType;
// use crate::tokenizer::position::Position;

pub struct Token<'a> {
    pub r#type: TType,
    pub start: Position,
    pub end: Position,
    pub text: &'a str,
}

#[allow(non_snake_case)]
impl<'a> Token<'a>  {

    pub(crate) fn Make(ttype: TType, start: Position, end: Position, content: & 'a str) -> Self {
        Self {
            r#type: ttype,
            start: start,
            end: end,
            text: content,
        }
    }

    pub fn quick(ttype: TType, line_no:usize, start_col:usize, end_col:usize, content: & 'a str) -> Self {
        Self {
            r#type: ttype,
            start: Position::t((start_col, line_no)),
            end: Position::t((end_col, line_no)),
            text: content,
        }
    }
}

impl<'a>  Debug for Token<'a>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.r#type)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("text", &self.text)
            .finish()
    }
}

impl <'a> Display for Token<'a>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.r#type)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("text", &self.text)
            .finish()
    }

}

impl <'a> PartialEq for Token<'a>  {
    fn eq(&self, other: &Self) -> bool {
        return self.r#type == other.r#type && self.text == other.text;
    }

    fn ne(&self, other: &Self) -> bool {
        return self.r#type != other.r#type || self.text != other.text;
    }
}