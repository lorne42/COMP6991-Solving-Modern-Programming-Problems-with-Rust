use std::io::{self, Write};
fn main() {
    let pattern_string = std::env::args()
        .nth(1)
        .expect("missing required command-line argument: <pattern>");

    let pattern = &pattern_string;

    // TODO: Replace the following with your code:
    loop {
        let mut line = String::new();

        // Read a line from stdin
        let bytes_read = io::stdin().read_line(&mut line).expect("Failed to read input");

        // If input ends (EOF) or an empty line is entered, exit loop
        if bytes_read == 0 || line.trim().is_empty() {
            break;
        }

        // Print the line if it contains the pattern
        if line.contains(&*pattern) {
            print!("{}", line); // Use print! to avoid double newlines
            io::stdout().flush().expect("Failed to flush output"); // Ensure immediate output
        }
    }
}
