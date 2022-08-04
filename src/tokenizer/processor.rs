
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
use std::ops::Add;
use log::{info};
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


static CAPTURE_QUOTE_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A"[^\n"\\]*(?:\\.[^\n"\\]*)*""#).expect("regex"));

static CAPTURE_APOS_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A'[^\n'\\]*(?:\\.[^\n'\\]*)*'"#).expect("regex"));

static SINGLE_QUOTE_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A""#).expect("regex"));

static SINGLE_QUOTE_STRING_CONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A"[.\n]?[^"]"#).expect("regex"));

static SINGLE_QUOTE_STRING_PRECONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A[^"]+""#).expect("regex"));

static SINGLE_APOSTROPHE_STRING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A'"#).expect("regex"));

static SINGLE_APOSTROPHE_PRECONTENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\A[^']?'"#).expect("regex"));


static SPACE_TAB_FORMFEED_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[ \f\t]+").expect("regex"));

static ANY_NON_NEWLINE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A[^\r\n]+").expect("regex"));

static STRING_PREFIX_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(?i)(u|[bf]r|r[bf]|r|b|f)").expect("regex"));

static POTENTIAL_IDENTIFIER_TAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\A(\w|[^\x00-\x7f])+").expect("regex"));
static POSSIBLE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A[a-zA-Z]{1}[\w\d]+").expect("regex"));
static POSSIBLE_ONE_CHAR_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A[a-zA-Z]{1}").expect("regex"));

static HEXNUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A0[xX](?:_?[0-9a-fA-F])+").expect("regex"));

static BINNUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A0[bB](?:_?[01])+").expect("regex"));

static OCTNUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A0[oO](?:_?[0-7])+").expect("regex"));

static DECNUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A(?:0(?:_?0)*|[1-9](?:_?[0-9])*)").expect("regex"));


#[derive(PartialEq, Debug)]
enum StringType {
    NONE,
    SINGLE,
    DOUBLE,
    TRIPLE,
}


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
    string_type: StringType,
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
            string_type: StringType::NONE,
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
                if self.string_type == StringType::TRIPLE {
                    if let Some(token) = self.process_triple_quote(&mut line) {
                        body.push(token);
                        self.string_continues = false;
                    }
                }

            }
            else if self.module.remaining() == 0 && line.len() == 1 {
                // TODO verifiy line[0] == '\n'
                if line.peek().expect("Missing last char!") == '\n' {
                    body.push(Token::quick(TType::EndMarker, module_size, 0, 0, ""));
                }

            }
            else if line.len() == 1 && line.peek().expect("last char") == '\n' {
                //Blank lines don't exist and don't have NEWLINE or NL endings
                continue;
            }
            else {
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
        }

        if self.paren_stack.len() > 0 {
            let (hopefully_last, _lineno) = self.paren_stack.pop().unwrap();
            return Err(TokError::UnmatchedClosingParen(hopefully_last));
        }

        if self.indent_stack.len() > 0 {
            while self.indent_stack.len() > 0 {
                self.indent_stack.pop();
                body.push(Token::quick(TType::Dedent, module_size+1, 0, 0, ""));

            }
        }

        if body.last().unwrap().r#type != TType::EndMarker {
            body.push(Token::quick(TType::EndMarker, module_size+1, 0, 0, ""));
        }


        return Ok(body);
    }

    fn process_line(&mut self, line: &mut ManagedLine) -> Result<Vec<Token>, TokError> {


        let mut product: Vec<Token> = Vec::new();
        let mut has_statement = false;

        let lineno = line.lineno.saturating_add(1);

        if self.string_continues == false {
        //Deal with empty lines first
            if line.text.len() == 1 || line.text == "\n" {

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


            let index = line.get_idx();

            //We're continuing to consume a string, like a `'` or `"` but it could also be a `"""`
            if self.string_continues == true {

                match self.string_type {
                    StringType::SINGLE => {
                        if let Some((current_idx, match_str)) = line.test_and_return(&SINGLE_APOSTROPHE_PRECONTENT){
                            self.string_buffer_content.push_str(match_str);
                            product.push(Token::Make(
                                TType::String,
                                self.string_start,
                                Position::t((current_idx, lineno)),
                                self.string_buffer_content.as_str()
                            ));
                            self.string_continues = false;
                        } else {
                            return Err(TokError::UnterminatedString);
                        }
                    },
                    StringType::DOUBLE => {
                        if let Some((current_idx, match_str)) = line.test_and_return(&SINGLE_QUOTE_STRING_PRECONTENT) {
                            self.string_buffer_content.push_str(match_str);
                            product.push(Token::Make(
                                TType::String,
                                self.string_start,
                                Position::t((current_idx, lineno)),
                                self.string_buffer_content.as_str()
                            ));
                            self.string_continues = false;
                        } else {
                            return Err(TokError::UnterminatedString);
                        }
                    },
                    StringType::TRIPLE => {
                        if let Some((current_idx, match_str)) = line.test_and_return(&TRIPLE_QUOTE_AND_PRECONTENT) {
                            self.string_buffer_content.push_str(match_str);
                            product.push(Token::Make(
                                TType::String,
                                self.string_start,
                                Position::t((current_idx, lineno)),
                                self.string_buffer_content.as_str()
                            ));
                            self.string_continues = false;
                            has_statement = true;
                        } else {
                            //Consume the whole line from current idx
                            self.string_buffer_content = format!("{}{}", self.string_buffer_content, line.return_all() );
                            return Ok(product);
                        }
                    },
                    _ => {
                        println!("How did i get here? {:?}", self.string_type);
                    }
                }

                continue;
            }
            //Look for a comment and consume all after it.
            else if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A#.*").expect("regex")) {
                product.push(
                            Token::quick(TType::Comment, lineno, index, current_idx, retval)
                );
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
                            Token::quick(TType::Op, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            // like Regex says, look for non-quoted strings
            else if let Some((current_idx, retval)) = line.test_and_return(&POSSIBLE_NAME.to_owned()) {
                product.push(
                         Token::quick(TType::Name, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            //Hex number
            else if let Some((current_idx, retval)) = line.test_and_return(&HEXNUMBER) {
                product.push(
                        Token::quick(TType::Number, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            //Binary number
            else if let Some((current_idx, retval)) = line.test_and_return(&BINNUMBER) {
                product.push(
                        Token::quick(TType::Number, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            //Octal number
            else if let Some((current_idx, retval)) = line.test_and_return(&OCTNUMBER) {
                product.push(
                        Token::quick(TType::Number, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }


            //Declarative number (has _'s)
            else if let Some((current_idx, retval)) = line.test_and_return(&DECNUMBER) {
                product.push(
                        Token::quick(TType::Number, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            // look for numbers
            else if let Some((current_idx, retval)) = line.test_and_return(&Regex::new(r"\A\d+").expect("regex")) {
                product.push(
                        Token::quick(TType::Number, lineno, index, current_idx, retval)
                    );
                has_statement = true;
            }
            //Absorb  any spaces
            else if let Some((_current_idx, _retval)) = line.test_and_return(&SPACE_TAB_FORMFEED_RE.to_owned()) {
            // pass/ignore WS - TODO REFACTOR!
            }
            //Look for line continuation
            else if Some('\\') == line.peek() {

                println!("TODO, deal with line continutations!");
                // self.string_continues = true;
                let _ = line.get();
            }

            // Seek and then handle """ tokens
            else if let Some((current_idx, match_str)) = line.test_and_return(&TRIPLE_QUOTE.to_owned()) {
                debug!("TQ3 matched on @ {},{}:{:?}", current_idx, lineno, match_str);

                self.string_continues = true;
                self.string_type = StringType::TRIPLE;
                self.string_buffer_content = format!("{}", match_str);
                self.string_start = Position::m(current_idx, lineno);

                if let Some((end_idx, end_match_str)) = line.test_and_return(&TRIPLE_QUOTE_AND_PRECONTENT) {
                    let str_content = format!(r#"""{}"#, end_match_str);
                    product.push(
                            Token::quick(TType::String, lineno, current_idx, end_idx, str_content.as_str())
                       );
                   has_statement = true;
                } else {
                    // Consume rest of the line!
                    self.string_buffer_content = format!("{}{}", self.string_buffer_content, line.return_all()  );
                }
            }

            else if let Some((current_idx, match_str)) = line.test_and_return(&POSSIBLE_ONE_CHAR_NAME) {
                //TODO peak for " or '

                product.push(Token::Make(
                    TType::Name,
                    Position::m(current_idx.saturating_sub(match_str.len()), lineno),
                    Position::m(current_idx, lineno),
                    match_str
                ));
            }
            else if let Some((current_idx, match_str)) = line.test_and_return(&CAPTURE_APOS_STRING) {
                product.push(Token::Make(
                    TType::String,
                    Position::m(current_idx, lineno),
                    Position::m(current_idx.saturating_add(match_str.len()), lineno),
                    match_str
                ));

            }
            else if let Some((current_idx, match_str)) = line.test_and_return( &CAPTURE_QUOTE_STRING) {
                product.push(Token::Make(
                    TType::String,
                    Position::m(current_idx, lineno),
                    Position::m(current_idx.saturating_add(match_str.len()), lineno),
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
                    println!("Did not capture: {:?} - #{}:{}", chr, lineno, line.idx);

                    return Err(TokError::BadCharacter(chr) );
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


    use crate::tokenizer::position::Position;
    use crate::tokenizer::ttype::TType;
    use crate::tokenizer::token::Token;

    macro_rules! test_token{
        ($token:expr, $ttype:expr, $content:expr)=>{
            assert_eq!($token.r#type, $ttype);
            assert_eq!($token.text, $content);
        }
    }

    macro_rules! test_token_w_position{
        ($token:expr, $ttype:expr, $start:expr, $end:expr, $content:expr)=>{

            assert_eq!($token.r#type, $ttype, "Testing for type with {:?} {:?} != {:?}", $token.text, $token.r#type, $ttype);
            assert_eq!($token.text, $content);
            assert_eq!($token.start, Position::t($start), "Testing for start with {:?} % {:?} : {:?} != {:?}", $token.text, $token.r#type, $token.start, $start);
            assert_eq!($token.end, Position::t($end), "Testing for end with {:?} % {:?} : {:?} != {:?}", $token.text, $token.r#type, $token.end, $end);

        }
    }


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


        assert_eq!(tokens[1].r#type, TType::String);
        assert_eq!(tokens[1].text, expected);
    }

    #[test]
    fn processor_properly_consumes_single_quote_strings_basic() {
        let mut engine = Processor::consume_file("test_fixtures/simple_string.py", Some("simple_string".to_string()));
        let tokens = engine.run(false).expect("Tokens");
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn processor_absorbs_multiline_triple_quoted_strings() {
        pretty_env_logger::init();


        println!("Loading multiline into processor");

        let tokens = Processor::tokenize_file("test_fixtures/multiline_strings.py", Some("multiline"), true);
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 3);

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

        let tokens = Processor::tokenize_file("test_fixtures/test_additive.py", Some("additive"), false);
        print_tokens(&tokens);
        test_token_w_position!(tokens[0], TType::Encoding, (0, 0), (0, 0), "utf-8" );
        test_token_w_position!(tokens[1], TType::Name, (0, 1), (1, 1), "x" );
        test_token_w_position!(tokens[2], TType::Op, (2, 1), (3, 1), "=" );
        test_token_w_position!(tokens[3], TType::Number, (4, 1), (5, 1), "1" );
        test_token_w_position!(tokens[4], TType::Op, (6, 1), (7, 1), "-" );
        test_token_w_position!(tokens[5], TType::Name, (8, 1), (9, 1), "y" );
        test_token_w_position!(tokens[6], TType::Op, (10, 1), (11, 1), "+" );
        test_token_w_position!(tokens[7], TType::Number, (12, 1), (14, 1), "15" );
        test_token_w_position!(tokens[8], TType::Op, (15, 1), (16, 1), "-" );
        test_token_w_position!(tokens[9], TType::Number, (17, 1), (18, 1), "1" );
        test_token_w_position!(tokens[10], TType::Op, (19, 1), (20, 1), "+" );
        test_token_w_position!(tokens[11], TType::Number, (21, 1), (26, 1), "0x124" );
        test_token_w_position!(tokens[12], TType::Op, (27, 1), (28, 1), "+" );
        test_token_w_position!(tokens[13], TType::Name, (29, 1), (30, 1), "z" );
        test_token_w_position!(tokens[14], TType::Op, (31, 1), (32, 1), "+" );
        test_token_w_position!(tokens[15], TType::Name, (33, 1), (34, 1), "a" );
        test_token_w_position!(tokens[16], TType::Op, (34, 1), (35, 1), "[" );
        test_token_w_position!(tokens[17], TType::Number, (35, 1), (36, 1), "5" );
        test_token_w_position!(tokens[18], TType::Op, (36, 1), (37, 1), "]" );
        test_token_w_position!(tokens[19], TType::Newline, (37, 1), (38, 1), "\n" );
        test_token_w_position!(tokens[20], TType::EndMarker, (0, 2), (0, 2), "" );
    }

    #[test]
    fn test_async() {
        let tokens = Processor::tokenize_file("test_fixtures/test_async.py", Some("test_async"), true);
        print_tokens(&tokens);

        assert_eq!(tokens.len(), 60);
        //fuckkkkkkkk me that's a lot of data to double check
    }

    #[test]
    fn test_comparison() {
        let tokens = Processor::tokenize_file("test_fixtures/test_comparison.py", Some("test_comparison"), false);
        test_token_w_position!(tokens[0], TType::Encoding, (0, 0), (0, 0), "utf-8" );
        test_token_w_position!(tokens[1], TType::Name, (0, 1), (2, 1), "if" );
        test_token_w_position!(tokens[2], TType::Number, (3, 1), (4, 1), "1" );
        test_token_w_position!(tokens[3], TType::Op, (5, 1), (6, 1), "<" );
        test_token_w_position!(tokens[4], TType::Number, (7, 1), (8, 1), "1" );
        test_token_w_position!(tokens[5], TType::Op, (9, 1), (10, 1), ">" );
        test_token_w_position!(tokens[6], TType::Number, (11, 1), (12, 1), "1" );
        test_token_w_position!(tokens[7], TType::Op, (13, 1), (15, 1), "==" );
        test_token_w_position!(tokens[8], TType::Number, (16, 1), (17, 1), "1" );
        test_token_w_position!(tokens[9], TType::Op, (18, 1), (20, 1), ">=" );
        test_token_w_position!(tokens[10], TType::Number, (21, 1), (22, 1), "5" );
        test_token_w_position!(tokens[11], TType::Op, (23, 1), (25, 1), "<=" );
        test_token_w_position!(tokens[12], TType::Number, (26, 1), (30, 1), "0x15" );
        test_token_w_position!(tokens[13], TType::Op, (31, 1), (33, 1), "<=" );
        test_token_w_position!(tokens[14], TType::Number, (34, 1), (38, 1), "0x12" );
        test_token_w_position!(tokens[15], TType::Op, (39, 1), (41, 1), "!=" );
        test_token_w_position!(tokens[16], TType::Number, (42, 1), (43, 1), "1" );
        test_token_w_position!(tokens[17], TType::Name, (44, 1), (47, 1), "and" );
        test_token_w_position!(tokens[18], TType::Number, (48, 1), (49, 1), "5" );
        test_token_w_position!(tokens[19], TType::Name, (50, 1), (52, 1), "in" );
        test_token_w_position!(tokens[20], TType::Number, (53, 1), (54, 1), "1" );
        test_token_w_position!(tokens[21], TType::Name, (55, 1), (58, 1), "not" );
        test_token_w_position!(tokens[22], TType::Name, (59, 1), (61, 1), "in" );
        test_token_w_position!(tokens[23], TType::Number, (62, 1), (63, 1), "1" );
        test_token_w_position!(tokens[24], TType::Name, (64, 1), (66, 1), "is" );
        test_token_w_position!(tokens[25], TType::Number, (67, 1), (68, 1), "1" );
        test_token_w_position!(tokens[26], TType::Name, (69, 1), (71, 1), "or" );
        test_token_w_position!(tokens[27], TType::Number, (72, 1), (73, 1), "5" );
        test_token_w_position!(tokens[28], TType::Name, (74, 1), (76, 1), "is" );
        test_token_w_position!(tokens[29], TType::Name, (77, 1), (80, 1), "not" );
        test_token_w_position!(tokens[30], TType::Number, (81, 1), (82, 1), "1" );
        test_token_w_position!(tokens[31], TType::Op, (82, 1), (83, 1), ":" );
        test_token_w_position!(tokens[32], TType::Newline, (83, 1), (84, 1), "\n" );
        test_token_w_position!(tokens[33], TType::Indent, (0, 2), (4, 2), "    " );
        test_token_w_position!(tokens[34], TType::Name, (4, 2), (8, 2), "pass" );
        test_token_w_position!(tokens[35], TType::Newline, (8, 2), (9, 2), "\n" );
        test_token_w_position!(tokens[36], TType::Dedent, (0, 3), (0, 3), "" );
        test_token_w_position!(tokens[37], TType::EndMarker, (0, 3), (0, 3), "" );
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
        //import sys, time
        // x = sys.modules['time'].time()

        let tokens = Processor::tokenize_file("test_fixtures/test_selector.py", Some("test_selector"), true);
        print_tokens(&tokens);

        test_token!(tokens[0], TType::Name, "import");
        test_token!(tokens[1], TType::Name, "sys");
        test_token!(tokens[2], TType::Op, ",");
        test_token!(tokens[3], TType::Name, "time");


        assert_eq!(tokens.len(), 19);
    }

    #[test]
    fn test_shift() {
        let tokens = Processor::tokenize_file("test_fixtures/test_shift.py", Some("test_shift"), false);
        test_token_w_position!(tokens[0], TType::Encoding, (0, 0), (0, 0), "utf-8" );
        test_token_w_position!(tokens[1], TType::Name, (0, 1), (1, 1), "x" );
        test_token_w_position!(tokens[2], TType::Op, (2, 1), (3, 1), "=" );
        test_token_w_position!(tokens[3], TType::Number, (4, 1), (5, 1), "1" );
        test_token_w_position!(tokens[4], TType::Op, (6, 1), (8, 1), "<<" );
        test_token_w_position!(tokens[5], TType::Number, (9, 1), (10, 1), "1" );
        test_token_w_position!(tokens[6], TType::Op, (11, 1), (13, 1), ">>" );
        test_token_w_position!(tokens[7], TType::Number, (14, 1), (15, 1), "5" );
        test_token_w_position!(tokens[8], TType::Newline, (15, 1), (16, 1), "\n" );
        test_token_w_position!(tokens[9], TType::EndMarker, (0, 2), (0, 2), "" );
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