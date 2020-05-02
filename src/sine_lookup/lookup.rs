use super::lookup_table::SIN_LOOKUP_TABLE;
use super::SCALING_FACTOR;

#[allow(dead_code)]
pub fn get_sine(degree: u32, value: i32) -> i32 {
    let degree = degree % 360;
    let value = value * SIN_LOOKUP_TABLE[degree as usize];
    value / SCALING_FACTOR as i32
}


//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sine_points() {
        let current = 1000;
        assert_eq!(0, get_sine(0, current));
        assert_eq!(current, get_sine(90, current));
        assert_eq!(0, get_sine(180, current));
        assert_eq!(-current, get_sine(270, current));
        assert_eq!(0, get_sine(360, current));
    }

    #[test]
    fn test_scaling() {
        use std::f32::consts::PI;
        let current = 10;
        assert_eq!(0, get_sine(0, current));
        assert_eq!(((0.25*PI).sin() * current as f32) as i32, get_sine(45, current));
        assert_eq!(0, get_sine(180, current));
        assert_eq!(((1.25*PI).sin() * current as f32) as i32, get_sine(180 + 45, current));
        assert_eq!(0, get_sine(360, current));
    }
}
