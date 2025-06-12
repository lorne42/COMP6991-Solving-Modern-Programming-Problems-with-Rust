use std::io::{self, Write};

const MAX_NAME_LEN: usize = 100;

fn main() {
    print!("What is your name? ");
    io::stdout().flush().unwrap(); // Flush output to ensure prompt is printed before input

    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read line");

    let name = name.trim(); // Remove newline and extra spaces

    // Truncate to MAX_NAME_LEN
    let name = &name[..name.len().min(MAX_NAME_LEN)];

    if name.is_empty() {
        println!("No name entered :(, goodbye.");
    } else {
        println!("Hello, {}, nice to meet you!", name);
    }
}
