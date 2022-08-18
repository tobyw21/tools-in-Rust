#![allow(unused)]

pub fn read_from_stdin() {
    
}

pub fn wc_default(content: &String, filename: &String) {
    let bytes = content.len() as i32;
    let mut lines = 0;
    let mut words = 0;
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

    println!(" {} {} {} {}", lines, words, bytes, filename);
}