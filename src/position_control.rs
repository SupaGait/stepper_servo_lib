//const DEGREES_PER_ENCODER_PULSE: i32 = 36;
const PULSES_PER_ROTATION: usize = 600 * 4;
//const CAL_SAMPLES_PER_STEP: usize = 4;

const ROTOR_TEETH: usize = 50;
const ROTOR_POLES: usize = 2;
const STEPS_PER_POLE: usize = 2; // Bipolar.
const STEPS_PER_ROTATION: usize = ROTOR_TEETH * ROTOR_POLES * STEPS_PER_POLE;

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

pub struct DebugCalibrationData {
    pub pulse_at_angle: [i32; PULSES_PER_ROTATION],
}

static mut DEBUG_CALIBRATION_DATA: DebugCalibrationData = DebugCalibrationData {
    pulse_at_angle: [0; PULSES_PER_ROTATION],
};

enum CalibrationPhase {
    Step1Backwards,
    Step2Forwards,
    Step3CalibratingForward,
    // Step4Wait,
    // Step5CalibratingBackward,
}
struct CalibrationData {
    slow_iteration: u32,
    current_step: u32,
    current_phase: CalibrationPhase,
    calibrated: bool,
    //position_data: [i32; PULSES_PER_ROTATION],
}
impl CalibrationData {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            slow_iteration: 0,
            current_step: 0,
            current_phase: CalibrationPhase::Step1Backwards,
            calibrated: false,
            //position_data: [0; PULSES_PER_ROTATION],
        }
    }
}

pub struct PositionControl<Input> {
    mode: Mode,
    calibration_data: CalibrationData,
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
            calibration_data: CalibrationData::default(),
            control_period,
            position_input,
            setpoint: 0,
            speed: 0,
            detected_angle: 0,
            angle_setpoint: 0,
            interpolation_change: 0,
        }
    }

    pub fn update_position(&mut self) {
        self.position_input.update();

        let position = self.position_input.get_position();
        let position = if position > 0 {
            position as usize % PULSES_PER_ROTATION
        } else {
            //PULSES_PER_ROTATION - ((position * -1) as usize % PULSES_PER_ROTATION)
            PULSES_PER_ROTATION - 1 - ((position % PULSES_PER_ROTATION as i32) * -1) as usize
        };

        unsafe {
            self.detected_angle = DEBUG_CALIBRATION_DATA.pulse_at_angle[position];
        }
        // match self.position_input.get_direction() {
        //     Direction::Increased(_value) => {
        //         self.detected_angle = self.detected_angle + DEGREES_PER_ENCODER_PULSE;
        //     }
        //     Direction::Decreased(_value) => {
        //         self.detected_angle = self.detected_angle - DEGREES_PER_ENCODER_PULSE;
        //     }
        //     _ => (),
        // }
        if let Mode::Calibration = self.mode {
            let position = self.position_input.get_position();
            if position >= 0 && position < PULSES_PER_ROTATION as i32 {
                unsafe {
                    DEBUG_CALIBRATION_DATA.pulse_at_angle[position as usize] = self.angle_setpoint;
                }
            }
        }
    }
    pub fn update(&mut self) {
        match self.mode {
            Mode::Normal => {
                self.calculate_next_angle();
            }
            Mode::Calibration => {
                self.calibrate();
            }
        }
    }

    fn calculate_next_angle(&mut self) {
        const COIL_PULL_ANGLE: i32 = 90;

        let position_diff = self.get_current_position() - self.setpoint;
        // @ 20kHz, 200steps
        // 20_000 / 360 = 55.5 => / 200 = 0.3 rotation
        // Move new angle based on speed
        if position_diff > 0 {
            //self.next_angle -= 1 * self.speed
            //self.interpolation_change -= 1;
            self.angle_setpoint = self.detected_angle - COIL_PULL_ANGLE;
        }
        if position_diff < 0 {
            //self.next_angle += 1 * self.speed
            //self.interpolation_change += 1;
            self.angle_setpoint = self.detected_angle + COIL_PULL_ANGLE;
        }

        if self.angle_setpoint < 0 {
            self.angle_setpoint += 360;
        }
        //self.next_angle = self.detected_angle + self.interpolation_change;
        self.angle_setpoint %= 360;
    }

    fn rotate_forwards(&mut self) {
        self.angle_setpoint = if self.angle_setpoint < 359 {
            self.angle_setpoint + 1
        } else {
            0
        };
    }

    fn rotate_backwards(&mut self) {
        if self.angle_setpoint > 0 {
            self.angle_setpoint -= 1;
        } else {
            self.angle_setpoint = 359;
        };
    }
    pub fn calibrate(&mut self) {
        //Slowly step through the full range and save angles.
        if self.calibration_data.slow_iteration < 10 {
            self.calibration_data.slow_iteration += 1;
        } else {
            self.calibration_data.slow_iteration = 0;

            match self.calibration_data.current_phase {
                // Expect angle = 359
                CalibrationPhase::Step1Backwards => {
                    self.rotate_backwards();
                    if self.angle_setpoint == 0 {
                        self.calibration_data.current_phase = CalibrationPhase::Step2Forwards;
                    }
                }
                CalibrationPhase::Step2Forwards => {
                    self.rotate_forwards();
                    if self.angle_setpoint == 0 {
                        self.position_input.reset();
                        self.calibration_data.reset();
                        self.calibration_data.current_phase =
                            CalibrationPhase::Step3CalibratingForward;
                    }
                }
                CalibrationPhase::Step3CalibratingForward => {
                    self.rotate_forwards();

                    // Each 90 degree is a step -> step at: 0, 90, 180, 270
                    if self.angle_setpoint % 90 == 0 {
                        self.calibration_data.current_step += 1;
                    }

                    // Are we done?
                    if self.calibration_data.current_step == STEPS_PER_ROTATION as u32 {
                        self.calibration_data.calibrated = true;
                        self.mode = Mode::Normal;
                        //self.calibration_data.current_phase = CalibrationPhase::Step4Wait;
                    }
                } // CalibrationPhase::Step4Wait => {
                  //     // Lets wait so the data can be retrieved.
                  // }
                  // CalibrationPhase::Step5CalibratingBackward => {
                  //     self.rotate_backwards();

                  //     // Are we done?
                  //     if self.calibration_data.current_step == 0 as u32 {
                  //         self.calibration_data.calibrated = true;
                  //         self.mode = Mode::Normal;
                  //     }

                  //     // Each 90 degree is a step -> step at: 0, 90, 180, 270
                  //     if self.angle_setpoint % 90 == 0 {
                  //         self.calibration_data.current_step -= 1;
                  //     }
                  // }
            }
        }
    }

    pub fn start_calibration(&mut self) {
        // if let Mode::Calibration = self.mode {
        //     // Continue
        //     self.calibration_data.current_phase = CalibrationPhase::Step5CalibratingBackward;
        // } else {
        // Reset
        self.angle_setpoint = 359;
        self.position_input.reset();
        self.calibration_data.reset();

        self.mode = Mode::Calibration;
        //}
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
    pub fn get_calibration_data(&self) -> &DebugCalibrationData {
        unsafe { &DEBUG_CALIBRATION_DATA }
    }
    pub fn calibration_is_done(&self) -> bool {
        self.calibration_data.calibrated
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
