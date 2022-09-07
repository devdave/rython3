extern crate core;
extern crate pretty_env_logger;

extern crate log;


mod tokenizer;
mod ast;
mod lexer;
mod parser;

use std::{env};

// use log::{debug, error, info};


use tokenizer::Processor;




fn main() {
    pretty_env_logger::init();


    let mut args = env::args();
    // if args.len() == 2 {
    //     let fname = args.nth(1).expect("Expected a filename");
    //
    //     println!("Processed {} and now tokenizing", fname);
    //     let mut engine = Processor::consume_file(fname, Some("__name__".to_string()));
    //
    //     let result = engine.run(false);
    //     if let Ok(retval) = result {
    //         for element in retval.iter() {
    //
    //             let token_range = format!("{},{}-{},{}:", element.start.line, element.start.col, element.end.line, element.end.col);
    //
    //             println!("{:20}  {:15?} '{:15?}'",
    //                      token_range,
    //                      element.r#type,
    //                      element.text);
    //
    //         }
    //     } else if let Err(retval) = result {
    //         error!("Main got a token error: {:?}", retval);
    //     }
    // } else {
    //     debug!("I got {} - {:?}", args.len(), args);
    //
    //
    //
    // }

}
