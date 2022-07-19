extern crate core;

mod tokenizer;
use std::{env};



use tokenizer::Processor;

// fn read_lines<P>(filename: P) -> Vec<String>
// where P: AsRef<Path>, {
//     let file = File::open(filename).unwrap();
//     io::BufReader::new(file).lines()
//         .map(|l| l.expect("Could not parse line"))
//         .collect()
//
//
//
// }

fn main() {
    println!("Hello, world!");
    let mut args = env::args();
    if args.len() == 2 {
        let mut engine = Processor::consume_file(args.nth(1).expect("Expected a file name as argument"), Some("__name__".to_string()));
        println!("runtime.exe file_to_tokenize.py");
        // let lines = read_lines(args.nth(1).unwrap());
        let result = engine.run();
        // let result = Processor::Consume_lines(lines);
        if let Ok(retval) = result {
            for element in retval.iter() {

                let token_range = format!("{},{}-{},{}:", element.lineno, element.col_start, element.line_end, element.col_end);

                println!("{:20}  {:15?} {:15?}",
                         token_range,
                         element.r#type,
                         element.text);

            }
        } else if let Err(retval) = result {
            println!("Main got a token error: {:?}", retval);
        }
    } else {
        println!("I got {} - {:?}", args.len(), args);



    }

}
