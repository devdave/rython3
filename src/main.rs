extern crate core;
extern crate pretty_env_logger;

#[macro_use] extern crate log;

mod tokenizer;
use std::{env};
use log::{debug, error, info};


use tokenizer::Processor;



fn main() {
    pretty_env_logger::init();


    let mut args = env::args();
    if args.len() == 2 {
        let mut engine = Processor::consume_file(args.nth(1).expect("Expected a file name as argument"), Some("__name__".to_string()));
        info!("runtime.exe file_to_tokenize.py");
        let result = engine.run();
        if let Ok(retval) = result {
            for element in retval.iter() {

                let token_range = format!("{},{}-{},{}:", element.line_start, element.col_start, element.line_end, element.col_end);

                println!("{:20}  {:15?} '{:15?}'",
                         token_range,
                         element.r#type,
                         element.text);

            }
        } else if let Err(retval) = result {
            error!("Main got a token error: {:?}", retval);
        }
    } else {
        debug!("I got {} - {:?}", args.len(), args);



    }

}
