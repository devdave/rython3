
use std::fmt::{Debug, Display, Formatter};
use std::io::empty;
use crate::tokenizer::position::Position;
use crate::tokenizer::ttype::TType;
// use crate::tokenizer::position::Position;

#[derive(Eq, Clone)]
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

    pub fn quick(ttype: TType, line_no:usize, start_col:usize, end_col:usize, tag_text: & 'a str) -> Self {
        Self {
            r#type: ttype,
            start: Position::t((start_col, line_no)),
            end: Position::t((end_col, line_no)),
            text: tag_text,
        }
    }

    pub fn quick_string(r#type: TType, line_no: usize, start_col: usize, end_col: usize, tag: String) -> Self {
        let mut temp = Self::quick(r#type, line_no, start_col, end_col, "");
        //attempt to get around E0515
        temp.text = &tag.as_str();
        return temp;
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