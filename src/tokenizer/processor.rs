
use std::cmp::Ordering;


use std::fs::File;


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
use log::{error, info};
use crate::tokenizer::position::Position;

// use std::str::Chars;

//Copied from LIBCST
//TODO relocate to a common rgxs.rs file?
const MAX_INDENT: usize = 100;
// const MAX_CHAR: char = char::MAX;
// const TAB_SIZE: usize = 8;
// // const ALT_TAB_SIZE: usize= 1;
// const SPACE_INDENT_SIZE: usize = 4;

static TRIPLE_QUOTE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A""""#).expect("regex"));

static TRIPLE_QUOTE_AND_CONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A"""[.\n]?"#).expect("regex"));

static TRIPLE_QUOTE_AND_PRECONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A.?""""#).expect("regex"));


static SINGLE_QUOTE_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A""#).expect("regex"));

static SINGLE_QUOTE_STRING_CONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A"[.\n]?[^"]"#).expect("regex"));

static SINGLE_QUOTE_STRING_PRECONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A[^"]+""#).expect("regex"));


static SPACE_TAB_FORMFEED_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[ \f\t]+").expect("regex"));

static ANY_NON_NEWLINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[^\r\n]+").expect("regex"));

static STRING_PREFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(?i)(u|[bf]r|r[bf]|r|b|f)").expect("regex"));

static POTENTIAL_IDENTIFIER_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(\w|[^\x00-\x7f])+").expect("regex"));
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
    string_start: Position,
    string_buffer_content: String,

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
            string_start: Position::default(),
            string_buffer_content: "".to_string(),
            module: ModuleLines::Make(lines, name),
        }
    }

    pub fn consume_file<P>(fname: P, module_name: Option<String>) -> Self
        where P: AsRef<std::path::Path>, {

        let mut buffer: String = String::new();
        let _file = File::open(fname).unwrap().read_to_string(&mut buffer);

        let lines = buffer.split(&NLSYM).map(|l| format!("{}\n", l).to_string()).collect();

        return Processor::initialize(lines, module_name);

    }

    pub fn tokenize_file<P>(fname: P, module_name: Option<&str>, skip_encoding: bool) -> Vec<Token>
        where P: AsRef<std::path::Path>,    {
        let mut engine = Processor::consume_file(fname, Some(module_name.unwrap().to_string()));
        return engine.run(skip_encoding).expect("tokens");

    }

    pub fn consume_string(input: String, module_name: Option<String>) -> Self {
        let product = if input.contains("\r\n") {
            input.split("\r\n")
        } else {
            input.split("\n")
        };

        let content = product.map(|l| format!("{}\n", l).to_string()).collect();

        info!("Processing string into Vector {:?}", content);
        return Processor::initialize(content, module_name);

    }

    pub fn consume_vector(input: Vec<String>, module_name: Option<String>) -> Self {
        return Processor::initialize(input, module_name);
    }

    pub fn run(&mut self, skip_encoding: bool) -> Result<Vec<Token>, TokError> {


        let mut body: Vec<Token> = Vec::new();

        // For now, ALWAYS assume UTF-8 encoding for files.
        if skip_encoding == false {
            body.push(Token::Make(TType::Encoding, Position::m(0,0), Position::m(0,0), "utf-8"));
        }

        let module_size = self.module.len();


        debug!("Starting walk/iterate over module");
        while self.module.has_lines() == true {



            let mut line = self.module.get().expect("Expected a line in module");
            debug!("Processing line: {:?}", line.text);

            if self.string_continues == true {
                debug!("inside of a string, consuming");

                if let Some(token) = self.process_triple_quote(&mut line) {
                    body.push(token);
                    self.string_continues = false;
                }
            }
            else if self.module.remaining() == 0 && line.len() == 1 {
                // TODO verifiy line[0] == '\n'
                if line.peek().expect("Missing last char!") == '\n' {
                    body.push(Token::Make(TType::EndMarker, Position::m(0, module_size), Position::m(0, module_size), ""));
                }

            } else {
                match self.process_line(&mut line) {
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

            }


        } // End While

        if self.string_continues == true {
            //We are out of lines
            return Err(TokError::UnterminatedTripleQuotedString);
            // panic!("Lines exhausted but string not closed!");
        }

        if self.paren_stack.len() > 0 {
            let (hopefully_last, _lineno) = self.paren_stack.pop().unwrap();
            return Err(TokError::UnmatchedClosingParen(hopefully_last));
        }

        if self.indent_stack.len() > 0 {
            while self.indent_stack.len() > 0 {
                self.indent_stack.pop();
                body.push(Token::Make(TType::Dedent, Position::m(0, module_size), Position::m(0, module_size),""));

            }
        }

        if body.last().unwrap().r#type != TType::EndMarker {
            body.push(Token::Make(TType::EndMarker, Position::m(0, module_size), Position::m(0, module_size), ""));
        }


        return Ok(body);
    }

    fn add_paren(&mut self, paren: char, lineno: usize) {
        self.paren_stack.push((paren, lineno));
    }

    fn process_line(&mut self, line: &mut ManagedLine) -> Result<Vec<Token>, TokError> {


        let mut product: Vec<Token> = Vec::new();
        let mut has_statement = false;

        let lineno = line.lineno;

        if self.string_continues == false {
        //Deal with empty lines first
            if line.text.len() == 0 || line.text == "" {
                // Assume ALL NL and Newlines are \n and not \r or \r\n - *nix or gtfo.


                if self.indent_stack.len() > 0 {
                    while self.indent_stack.len() > 0 {
                        self.indent_stack.pop();
                        product.push(Token::Make(TType::Dedent,  Position::m(0, lineno), Position::m(0, lineno), ""));
                    }
                }

                product.push(Token::Make(TType::NL, Position::m(0,lineno), Position::m(0,lineno, ), "\n"));

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
                        product.push(Token::Make(TType::Indent, Position::m(0, lineno), Position::m(current_size, lineno), whitespace.as_str()));
                    },
                    Ordering::Less => {
                        //We are handling 1 or more dedents
                        while self.indent_stack.len() > 0 {
                            let last_indent_size = self.indent_stack.pop().unwrap();
                            product.push(Token::Make(TType::Dedent, Position::m(0, lineno), Position::m(0, lineno), ""));
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
                    product.push(Token::Make(TType::Dedent, Position::m(0, lineno), Position::m(0, lineno),""));
                }
            }

            //Skip whitespace
            let mut _ignore_me = line.test_and_return(&SPACE_TAB_FORMFEED_RE.to_owned());

        }

        while line.peek() != None {

            //Look for a comment and consume all after it.
            if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A#.*").expect("regex")) {
                product.push(
                    Token::Make(TType::Comment,
                                Position::m( current_idx, lineno),
                                Position::m(retval.len()+current_idx, lineno),
                                &retval));
            }
            // Look for a operator
            else if let Some((current_idx, retval)) = line.test_and_return(&OPERATOR_RE.to_owned()) {
                let char_retval = retval.chars().nth(0).unwrap();

                if retval.len() == 1 && retval.contains(&['[', '(']) {
                    self.paren_stack.push((char_retval, lineno));


                } else if retval.contains(&[']', ')']) {
                    let latest = self.paren_stack.last();
                    match latest {
                        Some((verify_char, _ignore)) => {
                            if *verify_char == '(' && char_retval == ')' {
                                self.paren_stack.pop();
                            } else if *verify_char == '[' && char_retval == ']' {
                                self.paren_stack.pop();
                            } else {
                                return Err(TokError::MismatchedClosingParen(*verify_char, char_retval));
                            }
                        },
                        None => {
                            return Err(TokError::UnmatchedClosingParen(char_retval));
                        }
                    }
                }

                product.push(
                    Token::Make(TType::Op,
                                Position::m(current_idx, lineno),
                                Position::m(current_idx+retval.len(), lineno),
                                &retval));
                has_statement = true;
            }
            // like Regex says, look for non-quoted strings
            else if let Some((current_idx, retval)) = line.test_and_return(&POSSIBLE_NAME.to_owned()) {
                product.push(
                    Token::Make(TType::Name,
                                Position::m(current_idx, lineno),
                                Position::m(current_idx+retval.len(), lineno),
                                &retval));
                has_statement = true;
            }
            // look for numbers
            else if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A\d+").expect("regex")) {
                product.push(
                    Token::Make(TType::Number,
                                Position::m(current_idx, lineno),
                                Position::m(current_idx+retval.len(), lineno),
                                &retval));
                has_statement = true;
            }
            //Absorb  any spaces
            else if let Some((_current_idx, _retval)) = line.test_and_return(&SPACE_TAB_FORMFEED_RE.to_owned()) {
                // pass/ignore WS - TODO REFACTOR!
            } else if Some('\\') == line.peek() {

                println!("TODO, deal with line continutations!");
                // self.string_continues = true;
                let _ = line.get();
            }

            // Seek and then handle """ tokens
            else if let Some((current_idx, match_str)) = line.test_and_return(&TRIPLE_QUOTE_AND_CONTENT.to_owned()) {
                debug!("TQ3 matched on @ {},{}:{:?}", current_idx, lineno, match_str);

                self.string_continues = true;
                self.string_buffer_content = format!("{}", match_str);
                if let Some((end_idx, end_match_str)) = line.test_and_return(&TRIPLE_QUOTE_AND_PRECONTENT) {
                    let str_content = format!(r#""""{}"#, end_match_str);
                    product.push(
                       Token::Make(
                           TType::String,
                           Position::m(current_idx, lineno),
                           Position::m(end_idx, lineno),
                           str_content.as_str()
                       )
                   );
                   has_statement = true;
                } else {
                    // Consume rest of the line!
                    self.string_buffer_content = format!("{}{}", self.string_buffer_content, line.return_all()  );
                }
            }
            //See and handle single quote strings
            else if let Some((current_idx, match_str)) = line.test_and_return(&SINGLE_QUOTE_STRING.to_owned()) {
               println!("SQ matched @ {}:{} {:?}", lineno, current_idx, match_str);
               if let Some((end_idx, end_match_str)) = line.test_and_return(&SINGLE_QUOTE_STRING_PRECONTENT) {
                   println!("I found the end of the string - {}", end_match_str);
                   let str_content = format!(r#""{}"#, end_match_str);
                   product.push(
                       Token::Make(
                           TType::String,
                           Position::m(current_idx, lineno),
                           Position::m(end_idx, lineno),
                           str_content.as_str()
                       )
                   );
                   has_statement = true;
               }

            }
            else if let Some((current_idx, match_str)) = line.test_and_return(&POSSIBLE_NAME) {
                println!("Found a name {}", match_str);
                product.push(Token::Make(
                    TType::Name,
                    Position::m(current_idx, lineno),
                    Position::m(current_idx+match_str.len(), lineno),
                    match_str
                ));

            }
            else {
                let chr = line.get().unwrap();


                if chr == '\n' {
                    let what = if has_statement == true {
                        TType::Newline
                    } else {
                        TType::NL
                    };
                    product.push(Token::Make(
                            what,
                            Position::m(line.len()-1, lineno),
                            Position::m(line.len(), lineno),
                            "\n"
                        ));

                } else {
                    error!("Did not capture: {:?} - #{}", chr, lineno);
                    return Err(TokError::BadCharacter(chr));
                }



            }

        } // end while line peek






        Ok(product)

    }

    //Assumes that the python string has already started
    fn process_triple_quote(&mut self, line: &mut ManagedLine) -> Option<Token> {


        while line.peek() != None {
            if let Some((new_idx, match_str)) = line.test_and_return(&TRIPLE_QUOTE_AND_PRECONTENT) {
                debug!("Captured closing 3Q and content {:?}", match_str);

                self.string_buffer_content = format!("{}{}", self.string_buffer_content, match_str);

                let str_token = Token::Make(TType::String,
                                            self.string_start,
                                            Position::m(new_idx.saturating_sub(match_str.len()), line.lineno),
                                            self.string_buffer_content.as_str()
                );

                return Some(str_token);
            } else if let Some((new_idx, match_str)) = line.test_and_return(&TRIPLE_QUOTE) {
                self.string_buffer_content = format!("{}{}", self.string_buffer_content, match_str);

                let str_token = Token::Make(TType::String,
                                            self.string_start,
                                            Position::m(new_idx.saturating_sub(match_str.len()), line.lineno),
                                            self.string_buffer_content.as_str()
                );
                return Some(str_token);
            } else {
                self.string_buffer_content = format!("{}{}", self.string_buffer_content, line.get().unwrap());
            }
        }

        return None;



    }

}


#[cfg(test)]
mod tests {
    use crate::Processor;
    // use crate::tokenizer::module_lines::ModuleLines;


    use crate::tokenizer::ttype::TType;
    use crate::tokenizer::token::Token;


    fn print_tokens(tokens: &Vec<Token>) {
        println!("Got {} tokens", tokens.len());
        for (idx, token) in tokens.iter().enumerate() {
            println!("{}: {:?}", idx, token);
        }
    }

    #[test]
    fn rust_experiment() {
        let mut actual = "".to_string();
        actual.push('\n');
        assert_eq!(actual, "\n");
    }


    #[test]
    fn processor_works() {
        Processor::consume_string("Hello\nWorld".to_string(), Some("__test__".to_string()));
    }

    #[test]
    fn processor_does_basic_dentation() {
        let tokens = Processor::consume_file("test_fixtures/basic_indent.py", Some("__test__".to_string())).run(false).expect("Tokens");
        assert!(tokens.len() > 1);
        print_tokens(&tokens);
    }

    #[test]
    fn processor_does_adv_dentation() {
        let tokens = Processor::consume_file("test_fixtures/crazy_dents.py", Some("__test__".to_string())).run(false).expect("Expected vec<Tokens>");
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
    fn processor_correctly_handles_endmarker_vs_nl() {
        let mut engine = Processor::consume_file("test_fixtures/simple_string.py", Some("_simple_string_".to_string()));
        let tokens = engine.run(false).expect("Tokens");
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn processor_consumes_triple_strings_v2() {
        let data =
            r#"
"""
    This is a test!
"""
"#;
        let expected =
            r#""""
    This is a test!
""""#;


        let mut engine = Processor::consume_string(data.to_string(), Some("__test__".to_string()));
        let tokens = engine.run(false).expect("tokens");

        print_tokens(&tokens);


        assert_eq!(tokens[2].r#type, TType::String);
        assert_eq!(tokens[2].text, expected);
    }

    #[test]
    fn processor_properly_consumes_single_quote_strings_basic() {
        let mut engine = Processor::consume_file("test_fixtures/simple_string.py", Some("_simple_string_".to_string()));
        let tokens = engine.run(false).expect("Tokens");
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn processor_absorbs_multiline_triple_quoted_strings() {
        pretty_env_logger::init();


        println!("Loading multiline into processor");
        let mut engine = Processor::consume_file("test_fixtures/multiline_strings.py", Some("__multiline__".to_string()));

        let tokens = engine.run(false).expect("tokens");
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 6);
    }


    #[test]
    fn processor_consume_handles_names() {
        let mut processor = Processor::initialize(vec!["    def hello_world():".to_string()], Some("__test__".to_string()));

        let mut line = processor.module.get().expect("Atleast one line");

        let retval = processor.process_line(&mut line);
        let tokens = retval.unwrap();

        print_tokens(&tokens);

        assert_eq!(6, tokens.len());
        assert_eq!(tokens[0].r#type, TType::Indent);
        assert_eq!(tokens[1].r#type, TType::Name);
        assert_eq!(tokens[2].r#type, TType::Name);
        let test_types = vec!(TType::Indent, TType::Name, TType::Name, TType::Op, TType::Op, TType::Op);
        for (idx, test_type) in test_types.iter().enumerate() {
            assert_eq!(&tokens[idx].r#type, test_type);
        }
    }


    #[test]
    fn test_additive() {

        let tokens = Processor::tokenize_file("test_fixtures/test_additive.py", Some("additive"), true);
        print_tokens(&tokens);
    }

    #[test]
    fn test_async() {
        let tokens = Processor::tokenize_file("test_fixtures/test_async.py", Some("test_async"), true);
    }

    #[test]
    fn test_comparison() {
        let tokens = Processor::tokenize_file("test_fixtures/test_comparison.py", Some("test_comparison"), true);
    }

    #[test]
    fn test_float() {
        let tokens = Processor::tokenize_file("test_fixtures/test_float.py", Some("test_float"), true);
    }

    #[test]
    fn test_function() {
        let tokens = Processor::tokenize_file("test_fixtures/test_function.py", Some("test_function"), true);
    }

    #[test]
    fn test_int() {
        let tokens = Processor::tokenize_file("test_fixtures/test_int.py", Some("test_int"), true);
    }

    #[test]
    fn test_long() {
        let tokens = Processor::tokenize_file("test_fixtures/test_long.py", Some("test_long"), true);
    }

    #[test]
    fn test_method() {
        let tokens = Processor::tokenize_file("test_fixtures/test_method.py", Some("test_method"), true);
    }

    #[test]
    fn test_multiplicative() {
        let tokens = Processor::tokenize_file("test_fixtures/test_multiplicative.py", Some("test_multiplicative"), true );
    }

    #[test]
    fn test_selector() {
        let tokens = Processor::tokenize_file("test_fixtures/test_selector.py", Some("test_selector"), true);
    }

    #[test]
    fn test_shift() {
        let tokens = Processor::tokenize_file("test_fixtures/test_shift.py", Some("test_shift"), true);
    }

    #[test]
    fn test_string() {
        let tokens = Processor::tokenize_file("test_fixtures/test_string.py", Some("test_string"), true);

    }

    #[test]
    fn test_unary() {
        let tokens = Processor::tokenize_file("test_fixtures/test_unary.py", Some("test_unary"), true);
    }
}