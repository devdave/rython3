
use std::cmp::Ordering;


use std::fs::File;

use unicode_segmentation::UnicodeSegmentation;

use super::{
    token::Token,
    error::TokError,
    ttype::TType,
    operators::OPERATOR_RE,
    managed_line::ManagedLine,
    module_lines::ModuleLines,
};

use once_cell::sync::Lazy;
use regex::{Regex};
use std::io::{Read};
use crate::tokenizer::error::TokError::UnterminatedTripleQuotedString;

// use std::str::Chars;

//Copied from LIBCST
//TODO relcoate to a common rgxs.rs file?
const MAX_INDENT: usize = 100;
const MAX_CHAR: char = char::MAX;
const TAB_SIZE: usize = 8;
const ALT_TAB_SIZE: usize= 1;
const SPACE_INDENT_SIZE: usize = 4;

static  triple_quote: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"""""#).expect("regex"));

static  triple_quote_and_content: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""""[.\n]?"#).expect("regex"));

static  triple_quote_and_precontent: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[.\n]?""""#).expect("regex"));

static SPACE_TAB_FORMFEED_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[ \f\t]+").expect("regex"));

static ANY_NON_NEWLINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[^\r\n]+").expect("regex"));

static STRING_PREFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(?i)(u|[bf]r|r[bf]|r|b|f)").expect("regex"));

static POTENTIAL_IDENTIFIER_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A([a-zA-Z0-9_]|[^\x00-\x7f])+").expect("regex"));
// static DECIMAL_DOT_DIGIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A\.[0-9]").expect("regex"));
// static DECIMAL_TAIL_RE: Lazy<Regex> =
//     Lazy::new(|| Regex::new(r"\A[0-9](_?[0-9])*").expect("regex"));
// static HEXADECIMAL_TAIL_RE: Lazy<Regex> =
//     Lazy::new(|| Regex::new(r"\A(_?[0-9a-fA-F])+").expect("regex"));
// static OCTAL_TAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A(_?[0-7])+").expect("regex"));
// static BINARY_TAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A(_?[01])+").expect("regex"));
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
    string_continues: bool,
    string_continue_startline: usize,
    string_continue_buffer: Vec<String>,
    /**
    Not used, to be cut
    */
    line_blank: bool,
    pub module: ModuleLines,

}


#[allow(non_snake_case)]
impl Processor {
    pub fn initialize(lines: Vec<String>, module_name: Option<String>) -> Self {

        let name = module_name.unwrap_or("__main__".to_string());
        Self {
            indent_stack: Vec::new(),
            paren_stack: Vec::new(),
            last_line_was_blank: false,
            string_continues: false,
            string_continue_startline: 0,
            string_continue_buffer: Vec::new(),
            line_blank: false,
            module: ModuleLines::Make(lines, name),
        }
    }

    pub fn consume_file<P>(fname: P, module_name: Option<String>) -> Self
        where P: AsRef<std::path::Path>, {

        let mut buffer: String = String::new();
        let _file = File::open(fname).unwrap().read_to_string(&mut buffer);

        let lines = buffer.split(&NLSYM).map(|l| l.to_string()).collect();

        return Processor::initialize(lines, module_name);

    }

    pub fn consume_string(input: String, module_name: Option<String>) -> Self {
        let lines = if input.contains("\r\n") {
            input.split("\r\n").map(|l| l.to_string()).collect()
        } else {
            input.split("\n").map(|l| l.to_string()).collect()
        };

        println!("Processing string into Vector {:?}", lines);
        return Processor::initialize(lines, module_name);

    }

    pub fn consume_vector(input: Vec<String>, module_name: Option<String>) -> Self {
        return Processor::initialize(input, module_name);
    }

    pub fn run(&mut self) -> Result<Vec<Token>, TokError> {

        let mut body: Vec<Token> = Vec::new();

        // For now, ALWAYS assume UTF-8 encoding for files.
        body.push(Token::Make(TType::Encoding, 1, 0, 0, "utf-8"));

        // let error: TokError;

        let module_size = self.module.len();

        while self.module.has_lines() == true {


            let outcome = self.consume_line();

            match outcome {
                Ok(mut product) => {
                    // So.... yeah, not ideal but I needed a way to duplicate/copy all the elements
                    // of product so I can append them to body.
                    // TODO - Refactor so this is less stupid.
                    body.append(&mut product);

                },
                Err(issue) => {
                    panic!("Tokenizer failure: {:?}", issue);
                    // TODO figure out why the borrow checker freaks out on this line
                    // return Err(issue.clone());
                }
            }



        } // End While

        if self.paren_stack.len() > 0 {
            let (hopefully_last, _lineno) = self.paren_stack.pop().unwrap();
            return Err(TokError::UnmatchedClosingParen(hopefully_last));
        }

        if self.indent_stack.len() > 0 {
            while self.indent_stack.len() > 0 {
                self.indent_stack.pop();
                body.push(Token::Make(TType::Dedent, module_size-1, 0, 0, ""));

            }
        }

        if body.last().unwrap().r#type != TType::EndMarker {
            body.push(Token::Make(TType::EndMarker, self.module.len()-1, 0, 0, ""));
        }


        return Ok(body);
    }

    fn consume_line(&mut self) -> Result<Vec<Token>, TokError> {


        let mut line = self.module.get().expect("Expected a line in module");
        let mut product: Vec<Token> = Vec::new();

        let lineno = line.lineno;
        // let ltext = line.text.to_string();


        // if self.string_continues == true {
        //     //Switch off the current continuation flag
        //     self.string_continues = false;
        //     // and let the continuation handler decide if it is going to continue.
        //     return self.handle_string_continuation(line);
        //
        // }


        //Deal with empty lines first
        if line.text.len() == 0 || line.text == "" {
            // Assume ALL NL and Newlines are \n and not \r or \r\n - *nix or gtfo.


            if self.indent_stack.len() > 0 {
                while self.indent_stack.len() > 0 {
                    self.indent_stack.pop();
                    product.push(Token::Make(TType::Dedent, line.lineno, 0, 0, &line.text[..]));
                }
            }

            product.push(Token::Make(TType::NL, line.lineno, 0, 1,"\n"));

            return Ok(product);
        }

        //Consume the beginning of the line and handle indentations and dedentations
        let ws_match = SPACE_TAB_FORMFEED_RE.find(&line.text[..]);
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
                    product.push(Token::Make(TType::Indent, line.lineno, 0, current_size, whitespace.as_str()));
                },
                Ordering::Less => {
                    //We are handling 1 or more dedents
                    while self.indent_stack.len() > 0 {
                        let last_indent_size = self.indent_stack.pop().unwrap();
                        product.push(Token::Make(TType::Dedent, line.lineno, 0, current_size, ""));
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
                product.push(Token::Make(TType::Dedent, line.lineno, 0, 0, ""));
            }
        }

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
                self.string_continues = true;
                let _ = line.get();
            }
            else if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r#"^"{3}.*"#).expect("regex")) {

                println!("Found the start of a triple quoted string!");
                if let Ok(closed_str) = self.handle_triple_quote(current_idx, retval.to_string()) {
                    println!("I got the string {:?}", closed_str);
                    product.push(Token::Make(TType::String, lineno, current_idx, closed_str.len()+current_idx, closed_str.as_str()));
                } else {
                    return Err(TokError::UnterminatedTripleQuotedString);
                }


            }
            else {
                let chr = line.get().unwrap();
                println!("Did not capture: {:?} - #{}", chr, lineno);
            }

        }


        if has_statenent == true && self.string_continues == false {
            product.push(Token::Make(TType::Newline, lineno, line.get_idx(), line.get_idx()+1, "\n"));
        } else {
            product.push(Token::Make(TType::NL, lineno, line.get_idx(), line.get_idx()+1, "\n"));
        }


        Ok(product)

    }

    fn handle_triple_quote(&mut self, current_idx: usize, collected: String) -> Result<String, TokError> {
        println!("Attempting to process {}-{:?}", current_idx, collected);
        // is the collected line just the triple quotes?
        let mut str_collected = "".to_string();
        let mut working_line = collected;
        let triple_quote = r#"""""#;

        if working_line.trim().starts_with(triple_quote) {
            println!("working_line starts with triple quote");

            if working_line.trim().len() == 3 {
                // Lets grab the next line and move on to search for triple quote
                println!("Fetching next line");
                if let Some(module_line) = self.module.get() {
                    println!("Next line is {:?}", module_line);
                    working_line = module_line.text;
                    str_collected.push('\n');
                } else {
                    return Err(TokError::UnterminatedTripleQuotedString);
                }
            } else {

                working_line = working_line[3..].to_string();
                if working_line.trim().ends_with(triple_quote) {
                    println!("I found a close to my triple string! {:?}", working_line);
                    return Ok(working_line[..3].to_string());

                } else {
                    panic!("What do I have here? {:?}", working_line);
                }
            }

        }

        println!("Starting loop");
        loop {
            if working_line.trim().starts_with(triple_quote){
                // We are done!
                println!("loop - Working line starts with triple");
                return Ok(str_collected);
            }
            else if working_line.trim().ends_with(triple_quote) {
                str_collected = format!("{}{}", str_collected, working_line[..3].to_string());
                return Ok(str_collected);
            }else {
                str_collected = format!("{}{}", str_collected, working_line);
                if let Some(managed_line ) = self.module.get() {
                    working_line = managed_line.text.to_string();
                } else if str_collected.len() == 0 {
                    return Err(UnterminatedTripleQuotedString);
                } else {
                    return Ok(str_collected);
                }
            }

        }

    }

    ///Currently only focus on handling triple quote strings
    fn handle_string_continuation(&mut self, _: ManagedLine) -> Result<Vec<Token>, TokError> {


        Err(TokError::BadIdentifier("Not implemented"))
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
        Processor::consume_string("Hello\nWorld".to_string(), Some("__test__".to_string()));

    }

    #[test]
    fn processor_does_basic_dentation() {

        let tokens = Processor::consume_file("test_fixtures/basic_indent.py", Some("__test__".to_string())).run().expect("Tokens");
        assert!(tokens.len() > 1);
    }

    #[test]
    fn processor_does_adv_dentation() {

        let tokens = Processor::consume_file("test_fixtures/crazy_dents.py", Some("__test__".to_string())).run().expect("Expected vec<Tokens>");
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

    // #[test]
    // fn processor_consumes_triple_quote_strings() {
    //     let tokens = Processor::Consume_file("test_fixtures/multiline_strings.py").expect("Expected vec<Tokens");
    // }

    #[test]
    fn processor_consumes_triple_strings_v2 () {
        let data =
r#"
"""
    This is a test!
"""
"#;

        let tokens = Processor::consume_string(data.to_string(), Some("__test__".to_string())).run().expect("tokens");
        println!("Processor consumes triple strings v2 got {:?}", tokens);

        assert_eq!(tokens[2].r#type, TType::String);
        assert_eq!(tokens[2].text, "\n    This is a test!\n");


    }



    #[test]
    fn processor_consume_handles_names() {

        let mut processor = Processor::initialize(vec!["    def hello_world():".to_string()], Some("__test__".to_string()));

        let retval = processor.consume_line();
        let tokens = retval.unwrap();

        println!("I got {}, {:?}", tokens.len(), tokens);
        assert_eq!(7, tokens.len());
        assert_eq!(tokens[0].r#type, TType::Indent);

    }

    #[test]
    fn managed_line_works() {
        let mut line = ManagedLine::Make(1,"abc".to_string());
        assert_eq!(line.peek().unwrap(), 'a');
    }

    #[test]
    fn managed_line_gets_to_end() {
        let mut line = ManagedLine::Make(1,"abc".to_string());
        assert_eq!(line.get().unwrap(), 'a');
        assert_eq!(line.get().unwrap(), 'b');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
    }

    #[test]
    fn managed_line_goes_back() {
        let mut line = ManagedLine::Make(1,"abc".to_string());
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

        let mut sane = ManagedLine::Make(1,"()[]".to_string());
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