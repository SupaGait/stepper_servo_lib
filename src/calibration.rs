use crate::position_control::{PositionInput, PULSES_PER_ROTATION};

const ROTOR_TEETH: usize = 50;
const ROTOR_POLES: usize = 2;
const STEPS_PER_POLE: usize = 2; // Bipolar.
const STEPS_PER_ROTATION: usize = ROTOR_TEETH * ROTOR_POLES * STEPS_PER_POLE;

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
    Step4Wait,
    #[cfg(cal_hyst)]
    Step5CalibratingBackward,
}
pub struct Calibration {
    slow_iteration: u32,
    angle_setpoint: i32,
    current_step: u32,
    current_phase: CalibrationPhase,
    calibrated: bool,
    last_position: usize,
    //position_data: [i32; PULSES_PER_ROTATION],
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            slow_iteration: 0,
            angle_setpoint: 359,
            current_step: 0,
            current_phase: CalibrationPhase::Step1Backwards,
            calibrated: false,
            last_position: 0,
            //position_data: [0; PULSES_PER_ROTATION],
        }
    }
}

impl Calibration {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn angle_at_position(&self, position: usize) -> i32 {
        unsafe { DEBUG_CALIBRATION_DATA.pulse_at_angle[position] }
    }

    pub fn update_position(&mut self, position: usize, angle: i32) {
        // Update changes
        if position != self.last_position {
            self.last_position = position;
            unsafe {
                DEBUG_CALIBRATION_DATA.pulse_at_angle[position] = angle;
            }
        }
    }

    pub fn get_calibration_data(&self) -> &DebugCalibrationData {
        unsafe { &DEBUG_CALIBRATION_DATA }
    }

    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }

    pub fn requested_angle(&self) -> i32 {
        self.angle_setpoint
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

    pub fn update<T>(&mut self, position_input: &mut T)
    where
        T: PositionInput,
    {
        //Slowly step through the full range and save angles.
        if self.slow_iteration < 1 {
            self.slow_iteration += 1;
        } else {
            self.slow_iteration = 0;

            match self.current_phase {
                // Expect angle = 359
                CalibrationPhase::Step1Backwards => {
                    self.rotate_backwards();
                    if self.angle_setpoint == 0 {
                        self.current_phase = CalibrationPhase::Step2Forwards;
                    }
                }
                CalibrationPhase::Step2Forwards => {
                    self.rotate_forwards();
                    if self.angle_setpoint == 0 {
                        position_input.reset();
                        self.reset();
                        self.current_phase = CalibrationPhase::Step3CalibratingForward;
                    }
                }
                CalibrationPhase::Step3CalibratingForward => {
                    self.rotate_forwards();

                    // Each 90 degree is a step -> step at: 0, 90, 180, 270
                    if self.angle_setpoint % 90 == 0 {
                        self.current_step += 1;
                    }

                    // Are we done?
                    if self.current_step == STEPS_PER_ROTATION as u32 {
                        self.current_phase = CalibrationPhase::Step4Wait;
                    }
                }
                #[cfg(not(cal_hyst))]
                CalibrationPhase::Step4Wait => {
                    // No additional step, complete
                    self.calibrated = true;
                }
                #[cfg(cal_hyst)]
                CalibrationPhase::Step4Wait => {
                    // Lets wait so the data can be retrieved.
                }
                #[cfg(cal_hyst)]
                CalibrationPhase::Step5CalibratingBackward => {
                    self.rotate_backwards();

                    // Are we done?
                    if self.current_step == 0 as u32 {
                        self.calibrated = true;
                        self.mode = Mode::Normal;
                    }

                    // Each 90 degree is a step -> step at: 0, 90, 180, 270
                    if self.angle_setpoint % 90 == 0 {
                        self.current_step -= 1;
                    }
                }
            }
        }
    }
}
