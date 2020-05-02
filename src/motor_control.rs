pub trait CurrentDevice {
    fn set_current(&mut self, milli_amps: i32);
    fn current(&self) -> i32;
}

pub trait PositionControlled {
    fn set_angle(&mut self, degrees: i32);
    fn get_angle(&self) -> i32;
}

pub struct MotorControl<T1, T2> {
    current_control_coil_a: T1,
    current_control_coil_b: T2,
    angle_setpoint: i32,
}

impl<T1: CurrentDevice, T2: CurrentDevice> MotorControl<T1, T2> {
    pub fn new(coil_a: T1, coil_b: T2) -> Self {
        Self {
            current_control_coil_a: coil_a,
            current_control_coil_b: coil_b,
            angle_setpoint: 0,
        }
    }
    pub fn get_current_control_coil_a(&mut self) -> &mut T1 {
        &mut self.current_control_coil_a
    }
    pub fn get_current_control_coil_b(&mut self) -> &mut T2 {
        &mut self.current_control_coil_b
    }
}

impl<T1: CurrentDevice, T2: CurrentDevice> PositionControlled for MotorControl<T1, T2> {
    fn set_angle(&mut self, degrees: i32) {
        self.angle_setpoint = degrees % 360;
        // For test, bias on 180
        let current = (self.angle_setpoint - 180) / 2; // -90ma to + 90ma
        self.current_control_coil_a.set_current(current);
    }
    fn get_angle(&self) -> i32 {
        self.angle_setpoint
    }
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMotor {
        pub current: i32,
    }

    impl CurrentDevice for MockMotor {
        fn set_current(&mut self, milli_amps: i32) {
            self.current = milli_amps;
        }
        fn current(&self) -> i32 {
            self.current
        }
    }

    #[test]
    fn motor_pos_test() {
        let mut motor_control =
            MotorControl::new(MockMotor { current: 0 }, MockMotor { current: 0 });

        // Test the test
        motor_control.set_angle(0);
        assert_eq!(-180 / 2, motor_control.get_current_control().current());

        motor_control.set_angle(180);
        assert_eq!(0, motor_control.get_current_control().current());

        motor_control.set_angle(360);
        assert_eq!(-180 / 2, motor_control.get_current_control().current());
    }
}
