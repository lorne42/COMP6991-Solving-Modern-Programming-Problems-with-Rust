use serde::Deserialize;
use std::collections::VecDeque;
use std::io;

#[derive(Debug, Deserialize)]
enum Instruction {
    Set(i32),
    Left,
    Right,
    Reset,
}

#[derive(Debug)]
struct Light {
    // TODO: change me!
    left: Option<Box<Light>>,
    right: Option<Box<Light>>,
    brightness: i32,
}

impl Light {
    fn new() -> Self {
        Light {
            brightness: 0,
            left: None,
            right: None,
        }
    }

    fn set_brightness(&mut self, brightness: i32) {
        self.brightness = brightness;
    }
}

fn get_instructions_from_stdin() -> VecDeque<Instruction> {
    let mut instructions = String::new();
    io::stdin().read_line(&mut instructions).unwrap();
    ron::from_str(&instructions).unwrap()
}

fn collect_brightness_values(node: &Option<Box<Light>>, values: &mut Vec<i32>) {
    if let Some(n) = node {
        values.push(n.brightness);
        collect_brightness_values(&n.left, values);
        collect_brightness_values(&n.right, values);
    }
}

fn main() {
    let instructions = get_instructions_from_stdin();

    let mut light = Light {
        left: None,
        right: None,
        brightness: 0,
    };

    let mut current_node = &mut light;

    for instruction in instructions {
        match instruction {
            Instruction::Set(brightness) => {
                current_node.set_brightness(brightness);
            }
            Instruction::Left => {
                // If the left child doesn't exist, create one
                if current_node.left.is_none() {
                    current_node.left = Some(Box::new(Light::new()));
                }
                // Move to the left child
                current_node = current_node.left.as_mut().unwrap();
            }
            Instruction::Right => {
                // If the right child doesn't exist, create one
                if current_node.right.is_none() {
                    current_node.right = Some(Box::new(Light::new()));
                }
                // Move to the right child
                current_node = current_node.right.as_mut().unwrap();
            }
            Instruction::Reset => {
                // Reset to the root node
                current_node = &mut light;
            }
        }
    }

    let mut brightness_values = Vec::new();
    collect_brightness_values(&Some(Box::new(light)), &mut brightness_values);

    let total_brightness: i32 = brightness_values.iter().sum();
    let count = brightness_values.len();
    let average_brightness = if count > 0 {
        total_brightness / count as i32
    } else {
        0
    };

    // Output the result
    println!("{}", average_brightness);
}
