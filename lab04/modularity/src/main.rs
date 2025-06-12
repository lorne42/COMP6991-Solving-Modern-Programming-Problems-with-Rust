
use tribonacci::compute_tribonacci;

mod utils;
mod constants;
mod tribonacci;



fn main() {
    let shift_size = utils::first_argument();

    match compute_tribonacci(shift_size) {
        Ok(tribonacci) => {
            println!("Values: {:?}", tribonacci);

            let sum: u128 = tribonacci.iter().sum();
            println!("\nSum: {}", sum);
        }
        Err(e) => {
            println!("Error: {}", e.get_message());
        }
    }
}
