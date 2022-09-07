use std::fs::File;
use std::io::Read;
use std::vec::IntoIter;


// use super::code_module::CodeModule;
use super::code_line::CodeLine;
use super::NLTransformer::{ String2Vec, NLTransformer};
use crate::tokenizer::{Token, TokError, Position, operators::OPERATOR_RE};
use crate::tokenizer::TType;


use crate::tokenizer::patterns::{
    COMMENT, FLOATING_POINT, NAME_RE,

};

use super::state::LexerState;



struct Lexer<'a> {
    tokens: Vec<Token<'a>>,


}

impl <'a> Lexer<'a> {

    pub fn lex_file<P>(fname:P) -> Result<Vec<Token<'a>>, TokError>
    where P: AsRef<std::path::Path>,
    {
        let mut buffer = String::new();
        File::open(fname).expect("Failed to open file").read_to_string(&mut buffer);

        let temp_lines: Vec<String> = String2Vec(buffer);
        let mut lines: Vec<CodeLine> = Vec::new();

        let lines = temp_lines.iter().map(|el| CodeLine::new(el.as_str())).collect();

        return Lexer::process(lines, true);
    }

    pub fn process(mut lines: Vec<CodeLine<'a>>, skip_encoding:bool) -> Result<Vec<Token<'a>>,TokError> {

        let mut product: Vec<Token> = Vec::new();

        if skip_encoding == false {
            product.push(Token::quick(TType::Encoding, 0, 0, 0, "utf-8"));
        }

        let mut state = LexerState::new();

        for (lineno, mut line) in lines.iter_mut().enumerate() {
            match tokenize_line(line, lineno, &mut state) {
                Ok(mut tokens) => product.append(&mut tokens),
                Err(issue) => return Err(issue),
            }
        }

        //Just go ahead and put one in always
        product.push(Token::quick(TType::EndMarker, 0, 0, 0, ""));

        return Ok(product);
    }


}

fn tokenize_line<'a>(line: &mut CodeLine<'a>, lineno: usize, state: &mut LexerState) -> Result<Vec<Token<'a>>,TokError> {

        let mut product: Vec<Token> = Vec::new();
        let mut is_statement: bool = false;

        loop {

            if line.remaining() <= 0 {
                break;
            }

            let index = line.position();

            //TODO string consumption

            //Consume Comments
            if let Some((new_idx, retstr)) = line.return_match(COMMENT.to_owned()) {
                product.push(
                    Token::quick(TType::Comment, lineno, index, new_idx, retstr)
                );
            }
            //Consume floats
            // else if let Some((new_idx, retstr)) = line.return_match(FLOATING_POINT.to_owned()) {
            //     product.push(
            //         Token::quick_string(TType::Number, lineno, index, new_idx, retstr)
            //     )
            // }
            // //Consume operators
            // else if let Some((new_idx, retstr)) = line.return_match(OPERATOR_RE.to_owned()) {
            //     product.push(
            //     Token::quick_string(TType::Op, lineno, index, new_idx, retstr)
            //     );
            //     is_statement = true;
            // }
            // //Scan for name tokens
            // else if let Some((new_idx, retstr)) = line.return_match(NAME_RE.to_owned()) {
            //     //TODO look for parents and brackets
            //     product.push(
            //         Token::quick_string(TType::Name, lineno, index, new_idx, retstr)
            //     );
            //
            //     is_statement = true;
            // }
            else {
                if let Some(chr) = line.get() {
                    if chr == "\n" {
                        if is_statement == true {
                            product.push(
                                Token::quick(TType::Newline, lineno, 0, 0, "\n")
                            );
                        } else {
                            product.push(
                                Token::quick(TType::NL, lineno, 0, 0, "\n")
                            );
                        }

                    } else {
                        return Err(TokError::BadCharacter(chr.chars().nth(0).unwrap()));
                    }
                } else {
                    panic!("Reached end of line but there is no required new line!")
                }
            }


        } // end loop

        return Ok(product);

    }

#[cfg(test)]
mod test {
    use crate::lexer::lexer::Lexer;

    #[test]
    fn test_float() {
        let mut tokens = Lexer::lex_file("test_fixtures/test_float.py").expect("lexer");
        println!("I got {} tokens", tokens.len());
    }
}