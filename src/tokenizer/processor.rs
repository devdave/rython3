
use std::cmp::Ordering;
use std::fs::File;
use super::{
    token::Token,
    error::TokError,
    ttype::TType,
    operators::OPERATOR_RE,
};

use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{Read};
// use std::str::Chars;

//Copied from LIBCST
//TODO relcoate to a common rgxs.rs file?
const MAX_INDENT: usize = 100;
const MAX_CHAR: char = char::MAX;
const TAB_SIZE: usize = 8;
const ALT_TAB_SIZE: usize= 1;
const SPACE_INDENT_SIZE: usize = 4;

static SPACE_TAB_FORMFEED_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[ \f\t]+").expect("regex"));
static ANY_NON_NEWLINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[^\r\n]+").expect("regex"));
static STRING_PREFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(?i)(u|[bf]r|r[bf]|r|b|f)").expect("regex"));
static POTENTIAL_IDENTIFIER_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A([a-zA-Z0-9_]|[^\x00-\x7f])+").expect("regex"));
static DECIMAL_DOT_DIGIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A\.[0-9]").expect("regex"));
static DECIMAL_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[0-9](_?[0-9])*").expect("regex"));
static HEXADECIMAL_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(_?[0-9a-fA-F])+").expect("regex"));
static OCTAL_TAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A(_?[0-7])+").expect("regex"));
static BINARY_TAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A(_?[01])+").expect("regex"));
static POSSIBLE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A[a-zA-Z]{1}[\w\d]+").expect("regex"));


#[cfg(target_os = "macos")]
static NLSYM: str = "\r";

#[cfg(target_os = "linux")]
static NLSYM: str = "\n";

#[cfg(target_os = "windows")]
static NLSYM: &str = "\r\n";


///Lowest tier tokenizer, handles tokenizing line
///
pub struct Processor {

    /**
        number of elements is how far indented the code is
        individual elements is the size of the identation.

        i think it's insane to mix tabs and spaces.
    */
    indent_stack: Vec<usize>,
    /**
        (paren symbol, starting line no)
     */
    paren_stack: Vec<(char, usize)>,
    /**
        line to line state
    */
    last_line_was_blank: bool,

    /**
    Was the last line an open string or ( or something along those lines?
    */
    continues: bool,

    line_blank: bool,
}

struct LineErr {
    lineno: usize,
    colpos: usize,
    detail: String,

}

struct ManagedLine<'t> {
    pub idx: usize,
    text: &'t str,
    content: Vec<char>,
}

#[allow(non_snake_case)]
impl<'t> ManagedLine<'t> {

    fn Make(input: &'t str) -> Self {
        Self {
            idx: 0,
            text: input,
            content: input.chars().collect(),
        }
    }

    pub fn get_idx(&mut self) -> usize {
        self.idx
    }

    pub fn get(&mut self) -> Option<char> {

        if self.content.len() > self.idx {
            let retval = self.content[self.idx] as char;
            // let retval = self.content.get(self.idx).unwrap() as char;
            self.idx += 1;
            Some(retval)
        } else {
            None
        }
    }

    pub fn backup(&mut self) {
        self.idx -= 1;
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.content.len() > self.idx {
            Some(self.content[self.idx] as char)
        } else {
            None
        }
    }

    pub fn get_pos(&self) -> usize {
        return self.idx;
    }

    pub fn len(&self) -> usize {
        return self.content.len();
    }

    pub fn remaining(&self) -> usize {
        self.len() - self.get_pos()
    }

    pub fn test_and_return(&mut self, pattern: &Regex) -> Option<(usize, &str)> {

        let remaining = &self.text[self.idx..];
        let test = pattern.find(remaining);
        if let Some(found) = test {
            let len = found.end() - found.start();
            self.idx += found.start() + len;
            return Some((self.idx, found.as_str()));
        }
        None
    }
}

#[allow(non_snake_case)]
impl Processor {
    fn initialize() -> Self {
        Self {
            indent_stack: Vec::new(),
            paren_stack: Vec::new(),
            last_line_was_blank: false,
            continues: false,
            line_blank: false,
        }
    }

    pub fn Consume_file<P>(fname: P) -> Result<Vec<Token>, TokError<'static>>
        where P: AsRef<std::path::Path>, {

        let mut buffer: String = String::new();
        let _file = File::open(fname).unwrap().read_to_string(&mut buffer);

        let lines = buffer.split(&NLSYM).map(|l| l.to_string()).collect();

        return Processor::Consume_lines(lines);
    }

    pub fn Consume_lines(lines: Vec<String>) -> Result<Vec<Token>, TokError<'static>> {
        let mut engine = Processor::initialize();
        let mut body: Vec<Token> = Vec::new();

        // For now, ALWAYS assume UTF-8 encoding for files.
        body.push(Token::Make(TType::Encoding, 1, 0, 0, "utf-8"));

        for (lineno, line) in lines.into_iter().enumerate() {
            let line_vec = engine.consume_line(lineno+1, &line);
            if let Ok(mut product) = line_vec {
                body.append(&mut product);
            } else if let Err(issue) = line_vec {
                panic!("Got an error token from consume line {:?}", issue);
            }
        }

        if engine.paren_stack.len() > 0 {
            let (hopefully_last, lineno) = engine.paren_stack.pop().unwrap();
            return Err(TokError::UnmatchedClosingParen(hopefully_last));
        }






        return Ok(body);
    }

    fn consume_line(&mut self, lineno: usize, line: &str) -> Result<Vec<Token>, TokError> {

        let mut product: Vec<Token> = Vec::new();
        let raw: &str = &format!("{}", line);

        //Deal with empty lines first
        if raw.len() == 0 || raw == "" {
            // Assume ALL NL and Newlines are \n and not \r or \r\n - *nix or gtfo.


            if self.indent_stack.len() > 0 {
                while self.indent_stack.len() > 0 {
                    self.indent_stack.pop();
                    product.push(Token::Make(TType::Dedent, lineno, 0, 0, raw));
                }
            }

            product.push(Token::Make(TType::NL, lineno, 0, 0,"\n"));

            return Ok(product);
        }

        //Consume the beginning of the line and handle indentations and dedentations
        let ws_match = SPACE_TAB_FORMFEED_RE.find(&raw);
        if let Some(whitespace) = ws_match {
            let current_size = whitespace.end() - whitespace.start();
            let last_size = self.indent_stack.last().unwrap_or(&0);

            match current_size.cmp(last_size) {
                Ordering::Greater => {
                    //We are handing an indent
                    if self.indent_stack.len() + 1 > MAX_INDENT {
                        return Err(TokError::TooDeep);
                    }
                    self.indent_stack.push(current_size);
                    product.push(Token::Make(TType::Indent, lineno, 0, current_size, ""));
                },
                Ordering::Less => {
                    //We are handling 1 or more dedents
                    while self.indent_stack.len() > 0 {
                        let last_indent_size = self.indent_stack.pop().unwrap();
                        product.push(Token::Make(TType::Dedent, lineno, 0, current_size, ""));
                        if last_indent_size == current_size {
                            break;
                        }
                    }

                },
                Ordering::Equal => {
                    //No operation
                }
            }

        }
        else if self.indent_stack.len() > 0 {
            //Handle edge case where dedent has gone all the way to zero
            while self.indent_stack.len() > 0 {
                self.indent_stack.pop();
                product.push(Token::Make(TType::Dedent, lineno, 0, 0, ""));
            }
        }

        let mut line = ManagedLine::Make(raw);

        //Skip whitespace
        let mut _ignore_me = line.test_and_return(&SPACE_TAB_FORMFEED_RE.to_owned());
        let mut has_statenent = false;

        while line.peek() != None {


            //Look for a comment and consume all after it.
            if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A\#.*").expect("regex")) {
                product.push(Token::Make(TType::Comment, lineno, current_idx - &retval.len(), current_idx, &retval));
            }
            // Look for a operator
            else if let Some((current_idx, retval)) = line.test_and_return(&OPERATOR_RE.to_owned()) {

                let char_retval = retval.chars().nth(0).unwrap();

                if retval.len() == 1 && retval.contains(&['[','(']) {
                    self.paren_stack.push( (char_retval, lineno) );
                } else if retval.contains(&[']',')']) {
                    let last_paren = self.paren_stack.last();
                    match last_paren {
                        Some((verify_char, _ignore)) => {
                            if *verify_char == '(' && char_retval == ')' {
                                self.paren_stack.pop();
                            } else if *verify_char == '[' && char_retval == ']' {
                                self.paren_stack.pop();
                            }else {
                                return Err(TokError::MismatchedClosingParen(*verify_char, char_retval));
                            }
                        },
                        None => {
                            return Err(TokError::UnmatchedClosingParen(char_retval));
                        }
                    }
                }

                product.push(Token::Make(TType::Op, lineno, current_idx - &retval.len(), current_idx, &retval));
                has_statenent = true;


            }
            // like Regex says, look for non-quoted strings
            else if let Some((current_idx, strreturn)) = line.test_and_return(&POSSIBLE_NAME.to_owned()) {
                product.push(Token::Make(TType::Name, lineno, current_idx - strreturn.len(), current_idx, &strreturn));
                has_statenent = true;

            }
            // look for numbers
            else if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A[0-9\.]+").expect("regex")) {
                product.push(Token::Make(TType::Number, lineno, current_idx - retval.len(), current_idx, &retval));
                has_statenent = true;
            }
            else if let Some((_current_idx, _retval )) = line.test_and_return(&SPACE_TAB_FORMFEED_RE.to_owned()) {
                // pass/ignore WS
            } else if Some('\\') == line.peek() {
                self.continues = true;
                let _ = line.get();
            }

            else {
                let chr = line.get().unwrap();
                println!("Did not capture: {:?}", chr);
            }
        }


        if has_statenent == true && self.continues == false {
            product.push(Token::Make(TType::Newline, lineno, line.get_idx(), line.get_idx(), "\n"));
        } else {
            product.push(Token::Make(TType::NL, lineno, line.get_idx(), line.get_idx(), "\n"));
        }


        Ok(product)

    }

}


#[cfg(test)]
mod tests {

    use crate::Processor;
    use crate::tokenizer::operators::OPERATOR_RE;
    use crate::tokenizer::processor::ManagedLine;
    use crate::tokenizer::ttype::TType;


    #[test]
    fn processor_works() {
        Processor::initialize();
    }

    #[test]
    fn processor_does_basic_dentation() {

        let tokens = Processor::Consume_file("test_fixtures/basic_indent.py").expect("Expected vec<Tokens>");
        assert!(tokens.len() > 1);
    }

    #[test]
    fn processor_does_adv_dentation() {

        let tokens = Processor::Consume_file("test_fixtures/crazy_dents.py").expect("Expected vec<Tokens>");
        let mut indents = 0;
        let mut dedents = 0;
        for token in tokens.iter() {
            if token.r#type == TType::Indent {
                indents += 1;
            } else if token.r#type == TType::Dedent {
                dedents += 1
            }
        }

        assert!(tokens.len() > 1);
        assert_eq!(indents, dedents);
    }

    #[test]
    fn processor_consume_lines_handles_names() {

        let mut processor = Processor::initialize();
        let tokens = processor.consume_line(1, "    def hello_world():");

        println!("I got {:?}", tokens);
    }

    #[test]
    fn managed_line_works() {
        let mut line = ManagedLine::Make("abc");
        assert_eq!(line.peek().unwrap(), 'a');
    }

    #[test]
    fn managed_line_gets_to_end() {
        let mut line = ManagedLine::Make("abc");
        assert_eq!(line.get().unwrap(), 'a');
        assert_eq!(line.get().unwrap(), 'b');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
    }

    #[test]
    fn managed_line_goes_back() {
        let mut line = ManagedLine::Make("abc");
        assert_eq!(line.get().unwrap(), 'a');
        assert_eq!(line.get().unwrap(), 'b');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
        line.backup();
        assert_eq!(line.peek().unwrap(), 'c');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
        assert_eq!(line.get(), None);
    }

    #[test]
    fn managed_line_swallows_operators() {

        let mut sane = ManagedLine::Make("()[]");
        //Somewhat problematic here is the regex is still Lazy<Regex> at this point
        // so for now primestart it
        let (_current_idx, retval1) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();

        assert_eq!(retval1.len(), 1);
        assert_eq!(retval1, "(");

        let (_current_idx, retval2) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval2, ")");

        let (_current_idx, retval3) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval3, "[");

        let (_current_idx, retval4) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval4, "]");


    }


}