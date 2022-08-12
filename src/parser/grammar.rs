// use std::fmt::{Debug, Formatter, self};
use std::rc::Rc;

use crate::tokenizer::{Token};
use crate::tokenizer::TType::{self, String, Number, Name, Op as Operator, NL, EndMarker, Newline};

use peg::str::LineCol;
use peg::{parser, Parse, ParseElem, RuleResult};


#[derive(Debug)]
pub struct TokVec(Vec<Rc<Token>>);

impl std::convert::From<Vec<Token>> for TokVec {
    fn from(vec: Vec<Token>) -> Self {
        TokVec(vec.into_iter().map(Rc::new).collect())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseLoc {
    pub start_pos: LineCol,
    pub end_pos: LineCol,
}

impl std::fmt::Display for ParseLoc {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.start_pos.fmt(f)
    }
}


impl Parse for TokVec {
    type PositionRepr = ParseLoc;

    fn start<'input>(&'input self) -> usize {
        0
    }

    fn is_eof<'input>(&'input self, p: usize) -> bool {
        p >= self.0.len()
    }

    fn position_repr<'input>(&'input self, p: usize) -> Self::PositionRepr {
        let tok = self.0.get(p).unwrap_or_else(||self.0.last().unwrap());
        ParseLoc {
            start_pos: LineCol {
                line: tok.start.line,
                column: tok.start.col,
                offset: tok.start.col,
            },
            end_pos: LineCol {
                line: tok.end.line,
                column: tok.end.col,
                offset: tok.end.col,
            },
        }
    }
}

type TokenRef = Rc<Token>;

impl ParseElem for TokVec {
    type Element = TokenRef;

    fn parse_elem(&self, pos: usize) -> RuleResult<Self::Element> {
        match self.0.get(pos) {
            Some(tok) => RuleResult::Matched(pos+1, tok.clone()),
            None => RuleResult::Failed,
        }
    }
}

pub struct ValueNode {
    pub result: u32,
}

// impl std::Debug for ValueNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ValueNode")
//             .field("result", &self.result)
//             .finish()
//     }
//
// }

parser! {
    pub grammar python() for TokVec {


        //Starting rules

        pub rule file(name: &str) -> Module
        = traced(<_file(name)>)

        pub rule expression_input() -> Expression
        = traced(<e:star_expressions() tok(NL, "Newline") tok(EndMarker, "EOF") {e}> )

        pub rule statement_input() -> Statement
        = traced(<s:statement() tok(EndMarker, "EOF") {s}>)

        pub rule _file(name: &str) -> Module
        = s:statements()? eof:tok(EndMarker, "EOF") {
                make_module(name, s.unwrap_or_default(), eof)
        }

        pub rule fstring() -> FString
        = traced(<e:star_expressions()>) { make_fstring(e) }


        //General statements

        rule statements() -> Vec<Statement<'a>>
        = statement()+

        rule statement() -> Statement
        = c:compound_stmnt() { Statement::Compound(c) }
        / s:simple_stmnt() {
            Statement::Simple(make_simple_statement_lines(s))
        }

        rule simple_stmts() -> SimpleStatementParts
        = first_tok:&_ stmts:separated_trailer(<simple_stmt()>, <lit(";")>) nl:Tok(NL, "NEWLINE") {
            SimpleStatementParts {
                first_tok,
                first_statement: stmts.0,
                rest: stmts.1,
                last_semi: stmts.2,
                nl
            }
        }

        #[cache]
        rule simple_stmt() -> SmallStatement
        = assignment()
        / e:star_expressions() { SmallStatement::Expr(Expr { value: e, semicolon: None }) }
            / &lit("return") s:return_stmt() { SmallStatement::Return(s) }
            // this is expanded from the original grammar's import_stmt rule
            / &lit("import") i:import_name() { SmallStatement::Import(i) }
            / &lit("from") i:import_from() { SmallStatement::ImportFrom(i) }
            / &lit("raise") r:raise_stmt() { SmallStatement::Raise(r) }
            / lit("pass") { SmallStatement::Pass(Pass { semicolon: None }) }
            / &lit("del") s:del_stmt() { SmallStatement::Del(s) }
            / &lit("yield") s:yield_stmt() { SmallStatement::Expr(Expr { value: s, semicolon: None }) }
            / &lit("assert") s:assert_stmt() {SmallStatement::Assert(s)}
            / lit("break") { SmallStatement::Break(Break { semicolon: None })}
            / lit("continue") { SmallStatement::Continue(Continue { semicolon: None })}
            / &lit("global") s:global_stmt() {SmallStatement::Global(s)}
            / &lit("nonlocal") s:nonlocal_stmt() {SmallStatement::Nonlocal(s)}





        rule lit(lit:  &'static str) -> TokenRef
        = [t] {? if t.text == lit {Ok(t)} else {Err(lit)}}

        rule tok(tok: TType, err: &'static str) -> TokenRef
        = [t] {? if t.r#type == tok { Ok(t)} else {Err(err)} }


        rule traced<T>(e: rule<T>) -> T =
            &(_* {
                #[cfg(feature = "trace")]
                {
                    println!("[PEG_INPUT_START]");
                    println!("{}", input);
                    println!("[PEG_TRACE_START]");
                }
            })
            e:e()? {?
                #[cfg(feature = "trace")]
                println!("[PEG_TRACE_STOP]");
                e.ok_or("")
            }



    } //end python grammar
} //end parse!

fn make_addition(first: TokenRef, second: TokenRef) -> ValueNode{
    println!("{:?},{:?}", first, second);
    let a: u32 = first.text.parse().expect("number");
    let b: u32 = second.text.parse().expect("second number");
    return ValueNode{ result: a.saturating_add(b) };
}

#[cfg(test)]
mod tests {
    use crate::parser::grammar::{python, TokenRef, TokVec};
    use crate::tokenizer::Token;
    use crate::tokenizer::TType::{Op, Number};
    use std::rc::Rc;


    #[test]
    fn basic() {
        let data = TokVec(
            vec![
                Rc::new(Token::quick(Number, 1, 1, 2, "1")),
                Rc::new(Token::quick(Op, 1, 3, 4, "+")),
                Rc::new(Token::quick(Number, 1, 5, 6, "1")), ],
        );


        let result = python::statement(&data);
        match result {
            Ok(value) => println!("success {}", value.result),
            Err(issue) => println!("problem {}", issue),
        }

        assert_eq!(1, 2);


    }

}