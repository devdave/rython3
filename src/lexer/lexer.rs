use std::fs::File;
use std::io::Read;



use super::code_module::CodeModule;
use super::code_line::CodeLine;
use super::NLTransformer::{ String2Vec, NLTransformer};
use crate::tokenizer::{Token, TokError, Position, operators::OPERATOR_RE};
use crate::tokenizer::TType;


use crate::tokenizer::patterns::{
    COMMENT, FLOATING_POINT, NAME_RE,

};

use super::state::LexerState;







struct Lexer<'a> {
    module: CodeModule<'a>,


}

impl <'a> Lexer<'a> {

    pub fn lex_file<P>(fname:P) -> Self
    where P: AsRef<std::path::Path>,
    {
        let mut buffer = String::new();
        File::open(fname).expect("Failed to open file").read_to_string(&mut buffer);

        let lines: Vec<String> = String2Vec(buffer);

        Self {
            module: CodeModule::new(lines),
            string_body: "".to_string(),
        }


    }

    pub fn process(&mut self, skip_encoding:bool) -> Result<Vec<Token>,TokError> {

        let mut product: Vec<Token> = Vec::new();


        if skip_encoding == false {
            product.push(Token::quick(TType::Encoding, 0, 0, 0, "utf-8"));
        }


        for mut line in self.module {
            match self.process_line(&mut line) {
                Ok(mut tokens) => product.append(&mut tokens),
                Err(issue) => return Err(issue),
            }
        }

        //Just go ahead and put one in always
        product.push(Token::quick(TType::EndMarker, 0, 0, 0, ""));

        return Ok(product);
    }

    fn process_line(& mut self, line: &mut CodeLine) -> Result<Vec<Token>,TokError> {
        let mut product: Vec<Token> = Vec::new();
        let mut is_statement: bool = false;
        let mut lineno = 0;

        loop {

            if line.remaining() <= 0 {
                break;
            }

            let index = line.position();

            //TODO string consumption

            //Consume Comments
            if let Some((new_idx, retstr)) = line.return_match(COMMENT.to_owned()) {
                product.push(
                    Token::quick(TType::Comment, lineno, index, new_idx, retstr.as_str())
                );
            }
            //Consume floats
            else if let Some((new_idx, retstr)) = line.return_match(FLOATING_POINT.to_owned()) {
                product.push(
                    Token::quick(TType::Number, lineno, index, new_idx, retstr.as_str())
                )
            }
            //Consume operators
            else if let Some((new_idx, retstr)) = line.return_match(OPERATOR_RE.to_owned()) {
                product.push(
                Token::quick(TType::Op, lineno, index, new_idx, retstr.as_str())
                );
                is_statement = true;
            }
            //Scan for name tokens
            else if let Some((new_idx, retstr)) = line.return_match(NAME_RE.to_owned()) {
                //TODO look for parents and brackets
                product.push(
                    Token::quick(TType::Name, lineno, index, new_idx, retstr.as_str())
                );

                is_statement = true;
            }
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
                    panic!("Reached end of line but there is no ending char!")
                }
            }


        } // end loop

        return Ok(product);

    }

}

#[cfg(test)]
mod test {
    use crate::lexer::lexer::Lexer;

    #[test]
    fn test_float() {
        let mut lexer = Lexer::lex_file("test_fixtures/test_float.py");
        let tokens = lexer.process(true);
    }
}