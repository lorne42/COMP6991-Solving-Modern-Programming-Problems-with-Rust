//! # My Caesar Cipher Library
//! 
//! This library provides functionality to perform a basic Caesar cipher shift on strings.
//! It includes functions to encode text by shifting characters forward while maintaining case.
//! 
//! # Examples
//! 
//! ```
//! use doctor_who::caesar_shift;
//! assert_eq!(caesar_shift(Some(3), vec!["hello".to_string()]), vec!["khoor".to_string()]);
//! ```
//! 
//! The crate also defines constants for shift values and character boundaries.

/// The default shift value used for the Caesar cipher.
pub const DEFAULT_SHIFT: i32 = 5;
/// ASCII value of uppercase 'A'.
const UPPERCASE_A: i32 = 65;
/// ASCII value of lowercase 'a'.
const LOWERCASE_A: i32 = 97;
/// The size of the English alphabet.
const ALPHABET_SIZE: i32 = 26;

/// Applies a Caesar cipher shift to a list of strings.
/// 
/// # Parameters
/// * `shift_by` - An optional shift value; if `None`, `DEFAULT_SHIFT` is used.
/// * `lines` - A vector of strings to be encoded.
/// 
/// # Returns
/// A vector of strings with the encoded text.
/// 
/// # Examples
/// ```
/// use doctor_who::caesar_shift;
/// assert_eq!(caesar_shift(Some(3), vec!["abc".to_string()]), vec!["def".to_string()]);
/// ```
pub fn caesar_shift(shift_by: Option<i32>, lines: Vec<String>) -> Vec<String> {
    let shift_number = shift_by.unwrap_or(DEFAULT_SHIFT) % ALPHABET_SIZE;
    lines.iter().map(|line| shift(shift_number, line)).collect()
}

/// Applies a Caesar cipher shift to a single string.
fn shift(shift_by: i32, line: &String) -> String {
    line.chars().map(|c| shift_char(c, shift_by)).collect()
}

/// Shifts a character by the specified amount, wrapping within the alphabet.
fn shift_char(c: char, shift_by: i32) -> char {
    let ascii = c as i32;
    if ('A'..='Z').contains(&c) {
        (((ascii - UPPERCASE_A + shift_by) % ALPHABET_SIZE + ALPHABET_SIZE) % ALPHABET_SIZE + UPPERCASE_A) as u8 as char
    } else if ('a'..='z').contains(&c) {
        (((ascii - LOWERCASE_A + shift_by) % ALPHABET_SIZE + ALPHABET_SIZE) % ALPHABET_SIZE + LOWERCASE_A) as u8 as char
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caesar_shift() {
        assert_eq!(caesar_shift(Some(3), vec!["abc".to_string()]), vec!["def".to_string()]);
        assert_eq!(caesar_shift(Some(1), vec!["xyz".to_string()]), vec!["yza".to_string()]);
        assert_eq!(caesar_shift(None, vec!["Hello, World!".to_string()]), vec!["Mjqqt, Btwqi!".to_string()]);
    }
}
