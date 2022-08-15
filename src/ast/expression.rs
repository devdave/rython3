use std::{mem::swap, rc::Rc};

// Atomic nodes

pub struct Name<'a> {
    pub value: &'a str,
}

pub struct Integer<'a> {
    //Because it can be 1234 and 1_234 it must be stored as a string
    pub value: &'a str,
}

pub struct Float<'a> {
    pub value: &'a str,
}

pub struct Binary<'a> {
    pub value: &'a str,
}

pub struct Hexidecimal<'a> {
    pub value: &'a str,
}

pub struct Imaginary<'a> {
    pub value: &'a str,
}

pub struct Comparison<'a> {
    pub left: Box<Expression<'a>>,
    pub comparisons: Vec<ComparisonTarget<'a>>,
}


// Composite nodes

pub enum Expression<'a> {
    Name(Box<Name<'a>>),
    Ellipsis,
    Integer(Box<Integer<'a>>),
    Float(Box<Float<'a>>),
    Binary(Box<Binary<'a>>),
    Hexidecimal(Box<Hexidecimal<'a>>),
    Imaginary(Box<Imaginary<'a>>),
    Comparison(Box<Comparison<'a>>),



}
