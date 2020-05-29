use crate::calibration::{Calibration, DebugCalibrationData};

const PULSES_PER_ROTATION: usize = 600 * 4;

#[derive(Clone, Copy)]
pub enum Direction {
    Increased(i32),
    Decreased(i32),
    Unknown(i32),
}
pub trait PositionInput {
    fn update(&mut self);
    fn reset(&mut self);
    fn get_position(&self) -> i32;
    fn get_direction(&self) -> Direction;
}
enum Mode {
    Normal,
    Calibration,
}

pub struct PositionControl<Input> {
    mode: Mode,
    calibration: Calibration,
    control_period: i32,
    position_input: Input,
    setpoint: i32,
    speed: i32,
    detected_angle: i32,
    angle_setpoint: i32,
    interpolation_change: i32,
}
impl<Input> PositionControl<Input>
where
    Input: PositionInput,
{
    pub fn new(position_input: Input, control_period: i32) -> Self {
        Self {
            mode: Mode::Normal,
            calibration: Calibration::default(),
            control_period,
            position_input,
            setpoint: 0,
            speed: 0,
            detected_angle: 0,
            angle_setpoint: 0,
            interpolation_change: 0,
        }
    }

    pub fn angle(&self) -> i32 {
        self.angle_setpoint
    }
    pub fn set_position(&mut self, position: i32) {
        self.setpoint = position;
    }
    pub fn get_current_position(&self) -> i32 {
        self.position_input.get_position()
    }
    pub fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
    }
    pub fn update(&mut self) {
        match self.mode {
            Mode::Normal => {
                self.calculate_next_angle();
            }
            Mode::Calibration => {
                self.calibration.update(&mut self.position_input);
                if self.calibration.isCalibrated() {
                    self.mode = Mode::Normal;
                } else {
                    self.angle_setpoint = self.calibration.requestedAngle();
                }
            }
        }
    }

    pub fn update_position(&mut self) {
        self.position_input.update();

        let position = self.position_input.get_position();
        let position = if position > 0 {
            position as usize % PULSES_PER_ROTATION
        } else {
            PULSES_PER_ROTATION - 1 - ((position % PULSES_PER_ROTATION as i32) * -1) as usize
        };

        match self.mode {
            Mode::Normal => {
                self.detected_angle = self.calibration.angle_at_position(position);
            }
            Mode::Calibration => {
                let position = self.position_input.get_position();
                if position >= 0 && position < PULSES_PER_ROTATION as i32 {
                    self.calibration
                        .update_position(position as usize, self.angle_setpoint);
                }
            }
        }
    }

    fn calculate_next_angle(&mut self) {
        const COIL_MAX_PULL_ANGLE: i32 = 60;
        const HALF_COIL_MAX_PULL_ANGLE: i32 = COIL_MAX_PULL_ANGLE / 2;

        let position = self.get_current_position();
        let position_diff = position - self.setpoint;

        // Prevent ossilations, reduce the pull as we are close.
        let diff = position_diff.abs();
        let pull_angle = match diff {
            0 | 1 => 0,
            2..=HALF_COIL_MAX_PULL_ANGLE => diff * 2,
            _ => COIL_MAX_PULL_ANGLE,
        };

        // Change new angle acording to position
        self.angle_setpoint = if position_diff.is_positive() {
            self.detected_angle - pull_angle
        } else {
            self.detected_angle + pull_angle
        };

        if self.angle_setpoint.is_positive() {
            self.angle_setpoint %= 360;
        } else {
            // We need to wrap arround to max angle.
            self.angle_setpoint += 360;
        }
    }

    #[cfg(not(cal_hyst))]
    pub fn start_calibration(&mut self) {
        // Reset
        self.position_input.reset();
        self.calibration.reset();
        self.mode = Mode::Calibration;
    }

    #[cfg(cal_hyst)]
    pub fn start_calibration(&mut self) {
        if let Mode::Calibration = self.mode {
            // Continue
            self.calibration_data.current_phase = CalibrationPhase::Step5CalibratingBackward;
        } else {
            // Reset
            self.position_input.reset();
            self.calibration_data.reset();
            self.mode = Mode::Calibration;
        }
    }
    pub fn get_calibration_data(&self) -> &DebugCalibrationData {
        self.calibration.get_calibration_data()
    }
    pub fn calibration_is_done(&self) -> bool {
        self.calibration.isCalibrated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyInput {
        position: i32,
        direction: Direction,
    }
    impl PositionInput for DummyInput {
        fn update(&mut self) {}
        fn reset(&mut self) {
            todo!()
        }
        fn get_position(&self) -> i32 {
            self.position
        }
        fn get_direction(&self) -> Direction {
            self.direction
        }
    }

    #[test]
    fn position_positive_diff() {
        // Start at 0
        let input = DummyInput {
            position: 0,
            direction: Direction::Unknown(0),
        };
        let mut position_control = PositionControl::new(input, 10);

        // Request a new position
        position_control.set_position(500);
        position_control.update();

        let next_angle = position_control.angle();
        assert_eq!(0 + DEGREES_PER_ENCODER_PULSE, next_angle);
    }

    #[test]
    fn position_negative_diff() {
        // Start at 0
        let input = DummyInput {
            position: 1000,
            direction: Direction::Unknown(0),
        };
        let mut position_control = PositionControl::new(input, 10);

        // Request a new position
        position_control.set_position(500);
        position_control.update();

        let next_angle = position_control.angle();
        assert_eq!(360 - DEGREES_PER_ENCODER_PULSE, next_angle);
    }
}
