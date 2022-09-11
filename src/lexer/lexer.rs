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
    lines: Vec<String>,
    codes: Vec<CodeLine<'a>>,
    pub tokens: Vec<Token<'a>>,

    pub was_error: bool,
    pub issue: Option<TokError>,


}

impl <'a> Lexer<'a> {

    fn new() -> Self {
        Self {
            lines: Vec::new(),
            codes: Vec::new(),
            tokens: Vec::new(),

            was_error: false,
            issue: None,
        }
    }


    pub fn lex_file<P>(fname:P) -> Vec<String>
    where P: AsRef<std::path::Path>,
    {
        let mut buffer = String::new();
        File::open(fname).expect("Failed to open file").read_to_string(&mut buffer);

        let temp_lines: Vec<String> = String2Vec(buffer);

        return temp_lines;


    }

    pub fn TokenizeFile<P>(&'a mut self, fname:P) -> bool
    where P: AsRef<std::path::Path>
    {

        self.lines = Lexer::lex_file(fname);
        for line in self.lines.iter() {
            self.codes.push(CodeLine::new(line.as_str()));
        }
        let result = Lexer::process(&mut self.codes, true);
        self.was_error = match result  {
            Ok(tokens) => {
                self.tokens = tokens;
                false
            },
            Err(issue) => {
                self.issue = Some(issue);
                true
            }
        };

        return self.was_error;

    }

    pub fn process(mut lines: &mut Vec<CodeLine<'a>>, skip_encoding:bool) -> Result<Vec<Token<'a>>,TokError> {

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
                    Token::quick(TType::Comment, lineno, index, new_idx, &retstr)
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
        let mut lexer = Lexer::new();
        let mut status = lexer.TokenizeFile("test_fixtures/test_float.py");
        assert_eq!(lexer.was_error, true);
        println!("I got {} tokens", lexer.tokens.len());
    }
}