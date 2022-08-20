use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufRead};

struct FileInfo {
    lines: u32,
    words: u32,
    characters: u32,
    bytes: u32,
}

impl FileInfo {
    fn new<R: BufRead>(reader: R) -> Result<Self, io::Error> {
        let mut stats = FileInfo {
            lines: 0,
            words: 0,
            characters: 0,
            bytes: 0,
        };
        let mut prev_word = char::MAX;

        for ch in reader.bytes() {
            stats.bytes += 1;
            
            let character = ch? as char;
            if character.is_ascii() {
                stats.characters += 1;
            }

            if character.eq(&'\n') {
                stats.lines += 1;
            }

            if prev_word.is_ascii_whitespace() {
                if character.is_alphanumeric() || character.is_ascii_punctuation() {
                    stats.words += 1;
                }
            }
    
            prev_word = character;

        }

        if stats.bytes > 0 {
            stats.words += 1;
        }

        Ok(stats)
    }
    
    fn read_from_stdin() {
        let stdfd = io::stdin();
        let fd = BufReader::new(stdfd);
        let filestats = FileInfo::new(fd).expect("error on reading stdin");
        
        println!("  {}", filestats);
    }
}

impl std::fmt::Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.lines, self.words, self.bytes)
    
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();


    if args.len() < 2 {
        
        FileInfo::read_from_stdin();

    } else {

        for i in 1..args.len() {
            let filename = &args[i];
            let fd = BufReader::new(File::open(filename).expect("error reading file"));
            
            let filestats = FileInfo::new(fd).expect("error returning file stats");
            
            println!("  {} {}", filestats, filename);
            
        }

    }

}
