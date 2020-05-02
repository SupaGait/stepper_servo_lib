use crate::current_control::CurrentDevice;
use crate::sine_lookup;

pub struct Coil<T: CurrentDevice> {
    current_output: T,
    angle_setpoint: i32,
}

impl<T: CurrentDevice> Coil<T> {
    pub fn new(current_output: T) -> Self {
        Self {
            current_output,
            angle_setpoint: 0,
        }
    }
    pub fn set_angle(&mut self, degrees: i32) {
        self.angle_setpoint = degrees % 360;
        // For test, bias on 180
        let current = (self.angle_setpoint - 180) / 2; // -90ma to + 90ma
        self.current_output.set_current(current);
    }
    pub fn current_control(&mut self) -> &mut T {
        &mut self.current_output
    }
}


//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCurrentOutput {
        pub current: i32,
    }

    impl CurrentDevice for MockCurrentOutput {
        fn set_current(&mut self, milli_amps: i32) {
            self.current = milli_amps;
        }
        fn current(&self) -> i32 {
            self.current
        }
    }

    #[test]
    fn motor_pos_test() {
        let mut coil =
        Coil::new(MockCurrentOutput { current: 0 });

        // Test the test
        coil.set_angle(0);
        assert_eq!(-180 / 2, coil.current_control().current());

        coil.set_angle(180);
        assert_eq!(0, coil.current_control().current());

        coil.set_angle(360);
        assert_eq!(-180 / 2, coil.current_control().current());
    }
}
