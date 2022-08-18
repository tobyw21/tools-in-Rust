#![allow(unused)]

use std::env;
use std::fs;
use std::process;
use std::io::{self, Read};

fn read_from_stdin() -> (usize, u32, u32) {

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Error on reading from stdin");
    wc_default(&buffer)
    
}

fn wc_default(content: &String) -> (usize, u32, u32) {
    let bytes = content.len();
    let mut lines: u32 = 0;
    let mut words: u32 = 0;
    let mut prev_word = char::MAX;
    

    for l in content.chars() {
        if l.eq(&'\n') {
            lines += 1;
        }

        if prev_word.is_ascii_whitespace() {
            if l.is_alphanumeric() || l.is_ascii_punctuation() {
                words += 1;
            }
        }

        prev_word = l;
    }

    if content.len() > 0 {
        words += 1;
    }

    (bytes, lines, words)

}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut bytes: usize = 0;
    let mut lines: u32 = 0;
    let mut words: u32 = 0;

    if args.len() < 2 {
        (bytes, lines, words) = read_from_stdin();
        
        process::exit(0);

    } else {
        for i in 1..args.len() {
            let filename = &args[i];
        
            let content = fs::read_to_string(filename).expect("error reading file");
            
            (bytes, lines, words) = wc_default(&content);
        }

    }

    
}
