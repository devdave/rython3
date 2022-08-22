use std::rc::Rc;

use crate::tokenizer::Token;

type TokenRef<'a> = Rc<Token<'a>>;

pub struct AssignEqual<'a> {
    pub(crate) tok: TokenRef<'a>,
}



pub enum AugOp {
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    MatrixMultiplyAssign,
    DivideAssign,
    ModuloAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    LeftShiftAssign,
    RightShiftAssign,
    PowerAssign,
    FloorDivideAssign,
}

pub struct Dot { }

pub enum UnaryOp {
    Plus,
    Minus,
    BitInvert,
    Not,
}

pub enum BinaryOp {
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

pub struct BitOr {}

pub enum BooleanOp {
    And,
    Or,
}

pub struct ImportStar {}

pub struct Colon {

}

pub struct Comma {

}



pub enum CompOp {
    LessThan ,
    GreaterThan ,
    LessThanEqual ,
    GreaterThanEqual ,
    Equal ,
    NotEqual ,
    In,
    NotIn,
    Is,
    IsNot ,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Semicolon<'a> {

    pub(crate) tok: TokenRef<'a>,
}