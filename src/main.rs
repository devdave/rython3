extern crate core;

mod tokenizer;

use std::{env};
use std::io::{self, BufRead};
use std::fs::File;
use std::path::Path;

use tokenizer::Processor;

fn read_lines<P>(filename: P) -> Vec<String>
where P: AsRef<Path>, {
    let file = File::open(filename).unwrap();
    io::BufReader::new(file).lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()



}

fn main() {
    println!("Hello, world!");
    let mut args = env::args();
    if args.len() == 2 {
        println!("runtime.exe file_to_tokenize.py");
        let lines = read_lines(args.nth(1).unwrap());
        let result = Processor::Consume_lines(lines);
        if let Ok(retval) = result {
            println!("Main got {:?}", retval);
        } else if let Err(retval) = result {
            println!("Main got a token error: {:?}", retval);
        }
    } else {
        println!("I got {} - {:?}", args.len(), args);
    }

}
