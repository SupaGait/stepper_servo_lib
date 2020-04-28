pub trait Motor {
    fn set_current(&mut self, milli_amps: i32);
    fn current(&self) -> i32;
}

pub trait Position {
    fn set_angle(&mut self, degrees: i32);
    fn get_angle(&self) -> i32;
}

pub struct MotorControl<T> {
    motor: T,
    angle_setpoint: i32,
}

impl<T: Motor> MotorControl<T> {
    pub fn new(motor: T) -> Self {
        Self {
            motor,
            angle_setpoint: 0,
        }
    }
    pub fn get_motor(&mut self) -> &mut T {
        &mut self.motor
    }
}

impl<T: Motor> Position for MotorControl<T> {
    fn set_angle(&mut self, degrees: i32) {
        self.angle_setpoint = degrees % 360;
        // For test, bias on 180
        let current = (self.angle_setpoint - 180) / 2; // -90ma to + 90ma
        self.motor.set_current(current);
    }
    fn get_angle(&self) -> i32 {
        self.angle_setpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMotor {
        pub current: i32,
    }

    impl Motor for MockMotor {
        fn set_current(&mut self, milli_amps: i32) {
            self.current = milli_amps;
        }
        fn current(&self) -> i32 {
            self.current
        }
    }

    #[test]
    fn motor_pos_test() {
        let mut motor_control = MotorControl::new(MockMotor { current: 0 });

        // Test the test
        motor_control.set_angle(0);
        assert_eq!(-180 / 2, motor_control.get_motor().current());

        motor_control.set_angle(180);
        assert_eq!(0, motor_control.get_motor().current());

        motor_control.set_angle(360);
        assert_eq!(180 / 2, motor_control.get_motor().current());
    }
}
