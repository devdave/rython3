use std::rc::Rc;

use crate::tokenizer::Token;

type TokenRef<'a> = Rc<Token<'a>>;

pub struct AssignEqual<'a> {
    pub(crate) tok: TokenRef<'a>,
}

pub enum UnaryOperation {
    Plus,
    Minus,
    BitInvert,
    Not,
}

pub enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    FloorDivide,
    Modulo,
    Power,
    LeftShift,
    RightShift,
    BitOr,
    BitAnd,
    BitXor,
    MatrixMultiply,
}

pub enum BooleanOp {
    And,
    Or,
}