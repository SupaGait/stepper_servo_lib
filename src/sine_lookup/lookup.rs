use super::lookup_table::SIN_LOOKUP_TABLE;
use super::SCALING_FACTOR;

#[allow(dead_code)]
pub fn get_sine(degree: u32, value: i32) -> i32 {
    let value = value * SIN_LOOKUP_TABLE[degree as usize];
    value / SCALING_FACTOR as i32
}