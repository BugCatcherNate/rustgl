use std::env;
use std::fs;

fn main() {
    // --snip--


    let contents = fs::read_to_string("test.txt")
        .expect("Something went wrong reading the file");

    println!("With text:\n{}", contents);
}

