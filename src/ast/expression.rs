use std::{mem::swap, rc::Rc};
use super::op::{
    UnaryOperation, BinaryOperation, BooleanOp,
};

// Atomic nodes

pub struct Comma<'a> {

}

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

// Semi-atomic/more complex nodes

pub struct Comparison<'a> {
    // kind of surprised Rust lets me make this recursive/orobus pattern
    pub left: Box<Expression<'a>>,
    pub comparisons: Vec<ComparisonTarget<'a>>,
}

pub struct ComparisonTarget<'a> {
    pub operator: CompOp<'a>,
    pub comparator: Expression<'a>,
}



pub enum Element<'a> {
    Simple {
        value: Expression<'a>,
    },
    Starred(Box<StarredElement<'a>>),
}

pub struct StarredElement<'a> {
    pub value: Box<Expression<'a>>,
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
    UnaryOperation(Box<UnaryOperation>),
    BinaryOperation(Box<BinaryOperation>),
    BooleanOperation(Box<BooleanOperation<'a>>),
    Attribute(Box<Attribute<'a>>),
    Tuple(Box<Tuple<'a>>),
    Call(Box<Call<'a>>),
    GeneratorExp(Box<GeneratorExp<'a>>),
    ListComp(Box<ListComp<'a>>),
    SetComp(Box<SetComp<'a>>),
    DictComp(Box<DictComp<'a>>),
    List(Box<List<'a>>),
    Set(Box<Set<'a>>),
    Dict(Box<Dict<'a>>),
    Subscript(Box<Subscript<'a>>),
    StarredElement(Box<StarredElement<'a>>),
    IfExp(Box<IfExp<'a>>),
    Lambda(Box<Lambda<'a>>),
    Yield(Box<Yield<'a>>),
    Await(Box<Await<'a>>),
    SimpleString(Box<SimpleString<'a>>),
    ConcatenatedString(Box<ConcatenatedString<'a>>),
    FormattedString(Box<FormattedString<'a>>),
    NamedExpr(Box<NamedExpr<'a>>),

}



pub struct Attribute<'a> {
    pub value: Box<Expression<'a>>,
    pub attr: Name<'a>,
}

pub struct Tuple<'a> {
    pub elements: Vec<Element<'a>>,
}

pub struct Call<'a> {
    pub func: Box<Expression<'a>>,
    pub args: Vec<Arg<'a>>,
}

pub struct GeneratorExp<'a> {
    pub elt: Box<Expression<'a>>,
    pub for_in: Box<CompFor<'a>>,
}

pub struct CompFor<'a> {
    pub target: AssignTargetExpression<'a>,
    pub iter: Expression<'a>,
    pub ifs: Vec<CompIf<'a>>,
    pub inner_for_in: Option<Box<CompFor<'a>>>,
    pub asynchronous: Option<Asynchronous<'a>>,
}

pub struct CompIf<'a> {
    pub test: Expression<'a>,
    pub(crate) if_tok: TokenRef<'a>,
}

pub enum AssignTargetExpression<'a> {
    Name(Box<Name<'a>>),
    Attribute(Box<Attribute<'a>>),
    StarredElement(Box<StarredElement<'a>>),
    Tuple(Box<Tuple<'a>>),
    List(Box<List<'a>>),
    Subscript(Box<Subscript<'a>>),
}

pub struct Subscript<'a> {
    pub value: Box<Expression<'a>>,
    pub slice: Vec<SubscriptElement<'a>>,
}

pub enum BaseSlice<'a> {
    Index(Box<Index<'a>>),
    Slice(Box<Slice<'a>>),
}

pub struct Index<'a> {
    pub value: Expression<'a>,
}

pub struct Slice<'a> {
    pub lower: Option<Expression<'a>>,
    pub upper: Option<Expression<'a>>,
    pub step: Option<Expression<'a>>,
}

pub struct SubscriptElement<'a> {
    pub slice: BaseSlice<'a>,
}

pub struct ListComp<'a> {
    pub elt: Box<Expression<'a>>,
    pub for_in: Box<CompFor<'a>>,
}

pub struct SetComp<'a> {
    pub elt: Box<Expression<'a>>,
    pub for_in: Box<CompFor<'a>>,
}

pub struct DictComp<'a> {
    pub key: Box<Expression<'a>>,
    pub value: Box<Expression<'a>>,
    pub for_in: Box<CompFor<'a>>,
}

pub struct List<'a> {
    pub elements: Vec<Element<'a>>,
}

pub struct Set<'a> {
    pub elements: Vec<Element<'a>>,
}

pub struct Dict<'a> {
    pub elements: Vec<DictElement<'a>>,
}

pub enum DictElement<'a> {
    Simple {
        key: Expression<'a>,
        value: Expression<'a>,
    },
    Starred(StarredDictElement<'a>),
}

pub struct StarredDictElement<'a> {
    pub value: Expression<'a>,
}

pub struct IfExp<'a> {
    pub test: Box<Expression<'a>>,
    pub body: Box<Expression<'a>>,
    pub orelse: Box<Expression<'a>>,
}

pub struct Lambda<'a> {
    pub params: Box<Parameters<'a>>,
    pub body: Box<Expression<'a>>,
}

pub struct Parameters<'a> {
    pub params: Vec<Param<'a>>,
    pub star_arg: Option<StarArg<'a>>,
    pub kwonly_params: Vec<Param<'a>>,
    pub star_kwarg: Option<Param<'a>>,
    pub posonly_params: Vec<Param<'a>>,
    pub posonly_ind: Option<ParamSlash<'a>>,
}

pub struct Param<'a> {
    pub name: Name<'a>,
    pub annotation: Option<Annotation<'a>>,
    pub equal: Option<AssignEqual<'a>>,
    pub default: Option<Expression<'a>>,
}

pub struct ParamSlash<'a> {
    pub comma: Option<Comma<'a>>,
}

pub enum StarArg<'a> {
    Star(Box<ParamStar<'a>>),
    Param(Box<Param<'a>>),
}

pub struct Yield<'a> {
    pub value: Option<Box<YieldValue<'a>>>,
}

pub enum YieldValue<'a> {
    Expression(Box<Expression<'a>>),
    From(Box<From<'a>>),
}

pub struct From<'a> {
    pub item: Expression<'a>,
}

pub struct Await<'a> {
    pub expression: Box<Expression<'a>>,
}

pub struct SimpleString<'a> {
    /// The texual representation of the string, including quotes, prefix
    /// characters, and any escape characters present in the original source code,
    /// such as ``r"my string\n"``.
    pub value: &'a str,
}

pub struct ConcatenatedString<'a> {
    pub left: Box<String<'a>>,
    pub right: Box<String<'a>>,
}

pub enum String<'a> {
    Simple(SimpleString<'a>),
    Concatenated(ConcatenatedString<'a>),
    Formatted(FormattedString<'a>),
}

pub struct FormattedString<'a> {
    pub parts: Vec<FormattedStringContent<'a>>,
    pub start: &'a str,
    pub end: &'a str,
}

pub enum FormattedStringContent<'a> {
    Text(FormattedStringText<'a>),
    Expression(Box<FormattedStringExpression<'a>>),
}

pub struct FormattedStringText<'a> {
    pub value: &'a str,
}

pub struct FormattedStringExpression<'a> {
    pub expression: Expression<'a>,
    pub conversion: Option<&'a str>,
    pub format_spec: Option<Vec<FormattedStringContent<'a>>>,
    pub equal: Option<AssignEqual<'a>>,
}



pub struct NamedExpr<'a> {
    pub target: Box<Expression<'a>>,
    pub value: Box<Expression<'a>>,
}