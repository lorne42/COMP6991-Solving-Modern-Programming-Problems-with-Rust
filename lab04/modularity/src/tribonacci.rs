use crate::constants;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct TribonacciError(String);

impl TribonacciError {

    pub fn get_message(&self) -> &str {
        &self.0
    }
}

pub fn compute_tribonacci(size: Result<usize, ParseIntError>) -> Result<Vec<u128>, TribonacciError> {
    let size = size.map_err(|_| TribonacciError(constants::ERROR_MESSAGE.to_string()))?;

    let mut tribonacci = vec![1_u128; 3]; 

    for i in 3..size {
        tribonacci.push(tribonacci[i - 1] + tribonacci[i - 2] + tribonacci[i - 3]);
    }

    Ok(tribonacci)
}