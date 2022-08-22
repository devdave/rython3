
use std::rc::Rc;

use crate::tokenizer::Token;

use super::expression::{Arg, AssignTargetExpression, Asynchronous, Expression, From, Parameters, StarredElement, Tuple, List, Subscript, Name, NameOrAttribute };
use super::op::{ AugOp, AssignEqual, BitOr, ImportStar};

type TokenRef<'a> = Rc<Token<'a>>;

pub struct AugAssign<'a> {
    pub target: AssignTargetExpression<'a>,
    pub operator: AugOp,
    pub value: Expression<'a>,
}



pub enum CompoundStatement<'a> {
    FunctionDef(FunctionDef<'a>),
    If(If<'a>),
    For(For<'a>),
    While(While<'a>),
    ClassDef(ClassDef<'a>),
    Try(Try<'a>),
    TryStar(TryStar<'a>),
    With(With<'a>),
    Match(Match<'a>),
}

pub struct ClassDef<'a> {
    pub name: Name<'a>,
    pub body: Suite<'a>,
    pub bases: Vec<Arg<'a>>,
    pub keywords: Vec<Arg<'a>>,
    pub decorators: Vec<Decorator<'a>>,
}

pub struct FunctionDef<'a> {
    pub name: Name<'a>,
    pub params: Parameters<'a>,
    pub body: Suite<'a>,
    pub decorators: Vec<Decorator<'a>>,
    pub returns: Option<Annotation<'a>>,
    pub asynchronous: Option<Asynchronous,>,

}

pub struct For<'a> {
    pub target: AssignTargetExpression<'a>,
    pub iter: Expression<'a>,
    pub body: Suite<'a>,
    pub orelse: Option<Else<'a>>,
    pub asynchronous: Option<Asynchronous,>,
}

pub struct Global<'a> {
    pub names: Vec<NameItem<'a>>,
}

pub struct If<'a> {
    /// The expression that, when evaluated, should give us a truthy value
    pub test: Expression<'a>,
    // The body of this compound statement.
    pub body: Suite<'a>,

    /// An optional ``elif`` or ``else`` clause. ``If`` signifies an ``elif`` block.
    pub orelse: Option<Box<OrElse<'a>>>,
    pub is_elif: bool,
}


pub enum ImportNames<'a> {
    Star(ImportStar),
    Aliases(Vec<ImportAlias<'a>>),
}

// pub struct IndentedBlock<'a> {
//     /// Sequence of statements belonging to this indented block.
//     pub body: Vec<Statement<'a>>,
// }

pub struct Match<'a> {
    pub subject: Expression<'a>,
    pub cases: Vec<MatchCase<'a>>,
}

pub struct MatchAs<'a> {
    pub pattern: Option<MatchPattern<'a>>,
    pub name: Option<Name<'a>>,
}

pub struct MatchCase<'a> {
    pub pattern: MatchPattern<'a>,
    pub guard: Option<Expression<'a>>,
    pub body: Suite<'a>,
}

pub struct MatchClass<'a> {
    pub cls: NameOrAttribute<'a>,
    pub patterns: Vec<MatchSequenceElement<'a>>,
    pub kwds: Vec<MatchKeywordElement<'a>>,
}


pub struct MatchList<'a> {
    pub patterns: Vec<StarrableMatchSequenceElement<'a>>,
}

pub struct MatchKeywordElement<'a> {
    pub key: Name<'a>,
    pub pattern: MatchPattern<'a>,
}

pub struct MatchMapping<'a> {
    pub elements: Vec<MatchMappingElement<'a>>,
    pub rest: Option<Name<'a>>,
}

pub struct MatchMappingElement<'a> {
    pub key: Expression<'a>,
    pub pattern: MatchPattern<'a>,
}


pub enum MatchPattern<'a> {
    Value(MatchValue<'a>),
    Singleton(MatchSingleton<'a>),
    Sequence(MatchSequence<'a>),
    Mapping(MatchMapping<'a>),
    Class(MatchClass<'a>),
    As(Box<MatchAs<'a>>),
    Or(Box<MatchOr<'a>>),
}

pub struct MatchOr<'a> {
    pub patterns: Vec<MatchOrElement<'a>>,
}

pub struct MatchOrElement<'a> {
    pub pattern: MatchPattern<'a>,
    pub separator: Option<BitOr>,
}

pub struct MatchTuple<'a> {
    pub patterns: Vec<StarrableMatchSequenceElement<'a>>,
}

pub enum MatchSequence<'a> {
    MatchList(MatchList<'a>),
    MatchTuple(MatchTuple<'a>),
}

pub struct MatchSequenceElement<'a> {
    pub value: MatchPattern<'a>,
}

pub struct MatchSingleton<'a> {
    pub value: Name<'a>,
}

pub struct MatchStar<'a> {
    pub name: Option<Name<'a>>,
}



pub struct MatchValue<'a> {
    pub value: Expression<'a>,
}




pub struct NameItem<'a> {
    pub name: Name<'a>,
}

pub struct Nonlocal<'a> {
    pub names: Vec<NameItem<'a>>,
}


pub enum Statement<'a> {
    Simple(SimpleStatementLine<'a>),
    Compound(CompoundStatement<'a>),
}

pub enum Suite<'a> {
    IndentedBlock(IndentedBlock<'a>),
    SimpleStatementSuite(SimpleStatementSuite<'a>),
}


pub struct SimpleStatementLine<'a> {
    pub body: Vec<SmallStatement<'a>>,
}

pub struct SimpleStatementSuite<'a> {
    /// Sequence of small statements. All but the last statement are required to have
    /// a semicolon.
    pub body: Vec<SmallStatement<'a>>,
}

pub enum SmallStatement<'a> {
    Pass,
    //TODO double check that Python doesn't have named break/continues
    Break,
    Continue,
    Return(Return<'a>),
    Expr(Expr<'a>),
    Assert(Assert<'a>),
    Import(Import<'a>),
    ImportFrom(ImportFrom<'a>),
    Assign(Assign<'a>),
    AnnAssign(AnnAssign<'a>),
    Raise(Raise<'a>),
    Global(Global<'a>),
    Nonlocal(Nonlocal<'a>),
    AugAssign(AugAssign<'a>),
    Del(Del<'a>),
}

pub enum StarrableMatchSequenceElement<'a> {
    Simple(MatchSequenceElement<'a>),
    Starred(MatchStar<'a>),
}

pub struct Raise<'a> {
    pub exc: Option<Expression<'a>>,
    pub cause: Option<From<'a>>,
}

pub struct Return<'a> {
    pub value: Option<Expression<'a>>,
}

pub struct Try<'a> {
    pub body: Suite<'a>,
    pub handlers: Vec<ExceptHandler<'a>>,
    pub orelse: Option<Else<'a>>,
    pub finalbody: Option<Finally<'a>>,
}

pub struct TryStar<'a> {
    pub body: Suite<'a>,
    pub handlers: Vec<ExceptStarHandler<'a>>,
    pub orelse: Option<Else<'a>>,
    pub finalbody: Option<Finally<'a>>,
}

pub struct Expr<'a> {
    pub value: Expression<'a>,
}

pub struct AnnAssign<'a> {
    pub target: AssignTargetExpression<'a>,
    pub annotation: Annotation<'a>,
    pub value: Option<Expression<'a>>,
    pub equal: Option<AssignEqual<'a>>,
}

pub struct Annotation<'a> {
    pub annotation: Expression<'a>,
}

pub struct AsName<'a> {
    pub name: AssignTargetExpression<'a>,
}

pub struct Assert<'a> {
    pub test: Expression<'a>,
    pub msg: Option<Expression<'a>>,
}

pub struct Assign<'a> {
    pub targets: Vec<AssignTarget<'a>>,
    pub value: Expression<'a>,
}

pub struct AssignTarget<'a> {
    pub target: AssignTargetExpression<'a>,
}

pub struct Import<'a> {
    pub names: Vec<ImportAlias<'a>>,
}

pub struct ImportAlias<'a> {
    pub name: NameOrAttribute<'a>,
    pub asname: Option<AsName<'a>>,
}

pub struct ImportFrom<'a> {
    pub module: Option<NameOrAttribute<'a>>,
    pub names: ImportNames<'a>,
    pub relative: Vec<Dot>,
}

// pub enum NameOrAttribute<'a> {
//     N(Box<Name<'a>>),
//     A(Box<Attribute<'a>>),
// }

pub enum OrElse<'a> {
    Elif(If<'a>),
    Else(Else<'a>),
}


pub struct Attribute<'a> {
    pub value: Box<Expression<'a>>,
    pub attr: Name<'a>,
    pub dot: Dot,
}

pub struct Decorator<'a> {
    pub decorator: Expression<'a>,
}

pub struct Del<'a> {
    pub target: DelTargetExpression<'a>,
}

pub enum DelTargetExpression<'a> {
    Name(Box<Name<'a>>),
    Attribute(Box<Attribute<'a>>),
    Tuple(Box<Tuple<'a>>),
    List(Box<List<'a>>),
    Subscript(Box<Subscript<'a>>),
}

pub struct Dot { }

pub struct Else<'a> {
    pub body: Suite<'a>,
}

pub struct ExceptHandler<'a> {
    pub body: Suite<'a>,
    pub r#type: Option<Expression<'a>>,
    pub name: Option<AsName<'a>>,
}

pub struct ExceptStarHandler<'a> {
    pub body: Suite<'a>,
    pub r#type: Expression<'a>,
    pub name: Option<AsName<'a>>,
}

pub struct Finally<'a> {
    pub body: Suite<'a>,
}

pub struct While<'a> {
    pub test: Expression<'a>,
    pub body: Suite<'a>,
    pub orelse: Option<Else<'a>>,
}

pub struct With<'a> {
    pub items: Vec<WithItem<'a>>,
    pub body: Suite<'a>,
    pub asynchronous: Option<Asynchronous,>,
}

pub struct WithItem<'a> {
    pub item: Expression<'a>,
    pub asname: Option<AsName<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct IndentedBlock<'a> {
    /// Sequence of statements belonging to this indented block.
    pub body: Vec<Statement<'a>>,

    /// A string represents a specific indentation. A ``None`` value uses the modules's
    /// default indentation. This is included because indentation is allowed to be
    /// inconsistent across a file, just not ambiguously.

    pub indent: Option<&'a str>,


    pub(crate) newline_tok: TokenRef<'a>,
    pub(crate) indent_tok: TokenRef<'a>,
    pub(crate) dedent_tok: TokenRef<'a>,
}