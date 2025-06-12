use bmp::{Image, Pixel, open};
use std::env;

fn main() {
    // Get command-line arguments and skip the first one (program name)
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: cargo run -- <path-to-bmp-files>");
        return;
    }

    for (i, file_path) in args.iter().enumerate() {

        println!("===== {} =====", file_path);

        match open(file_path) {
            Ok(image) => print_image(&image),
            Err(e) => println!("Error! {:?}", e),
        }
    }
}

// Function to print the BMP image as ASCII representation
fn print_image(image: &Image) {
    let width = image.get_width();
    let height = image.get_height();

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            print!("{} ", map_pixel_to_char(pixel));
        }
        println!(); // New line at the end of each row
    }
}

// Function to map BMP pixel colors to characters
fn map_pixel_to_char(pixel: Pixel) -> &'static str {
    match (pixel.r, pixel.g, pixel.b) {
        (255, 0, 0) => "R",   // Red
        (0, 255, 0) => "G",   // Lime (Green)
        (0, 0, 255) => "B",   // Blue
        (255, 255, 255) => "W", // White
        _ => "?",  // Unknown color
    }
}
