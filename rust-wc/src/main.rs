use std::env;
use std::fs;
use std::process;

mod helper;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)

    let content = fs::read_to_string(filename).expect("error reading file");

    
}
