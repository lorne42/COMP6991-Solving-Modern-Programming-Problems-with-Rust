use bmp::{Image, Pixel};

fn main() {
    let width = 200;
    let height = 200;

    // Ensure the image has valid dimensions
    assert!(width > 0 && height > 0, "Image dimensions must be greater than zero");

    let mut img = Image::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let r = (x % 256) as u8;         // Red channel varies with x
            let g = (y % 256) as u8;         // Green channel varies with y
            let b = ((x + y) % 256) as u8;   // Blue channel varies diagonally

            img.set_pixel(x, y, Pixel::new(r, g, b));
        }
    }

    // Save the generated image as "output.bmp"
    let _ = img.save("output.bmp").expect("Failed to save image");
    
    println!("Image generated: output.bmp");
}
