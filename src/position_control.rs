//const DEGREES_PER_ENCODER_PULSE: i32 = 36;
const PULSES_PER_ROTATION: usize = 600 * 4;
const DEGREES_PER_ENCODER_PULSE: i32 = 36;

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

pub struct Debug_calibration_data {
    pub pulse_at_angle: [i32; PULSES_PER_ROTATION],
    pub position_at_cal_step: [[i32; 4]; 100],
}

static mut DEBUG_CALIBRATION_DATA: Debug_calibration_data = Debug_calibration_data {
    pulse_at_angle: [0; PULSES_PER_ROTATION],
    position_at_cal_step: [[0; 4]; 100],
};

//static mut PULSE_AT_ANGLE: [i32; PULSES_PER_ROTATION] = [0; PULSES_PER_ROTATION];
//static mut ANGLE_AT_CAL_STEP: [[i32; 5]; 200] = [[0; 5]; 200];

struct CalibrationData {
    slow_iteration: u32,
    current_step: u32,
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
        match self.position_input.get_direction() {
            Direction::Increased(_value) => {
                self.detected_angle = self.detected_angle + DEGREES_PER_ENCODER_PULSE;
            }
            Direction::Decreased(_value) => {
                self.detected_angle = self.detected_angle - DEGREES_PER_ENCODER_PULSE;
            }
            _ => (),
        }
        // if let Mode::Calibration = self.mode {
        //     let position = self.position_input.get_position();
        //     if position >= 0 && position < PULSES_PER_ROTATION as i32 {
        //         self.calibration_data.position_data[position as usize] = self.detected_angle;
        //     }
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
        let position_diff = self.get_current_position() - self.setpoint;

        // @ 20kHz, 200steps
        // 20_000 / 360 = 55.5 => / 200 = 0.3 rotation
        // Move new angle based on speed
        if position_diff > 0 {
            //self.next_angle -= 1 * self.speed
            //self.interpolation_change -= 1;
            self.angle_setpoint = self.detected_angle - DEGREES_PER_ENCODER_PULSE;
        }
        if position_diff < 0 {
            //self.next_angle += 1 * self.speed
            //self.interpolation_change += 1;
            self.angle_setpoint = self.detected_angle + DEGREES_PER_ENCODER_PULSE;
        }
        //self.next_angle = self.detected_angle + self.interpolation_change;
        self.angle_setpoint %= 360;
    }

    fn calibrate(&mut self) {
        //Slowly step through the full range and save angles.
        if self.calibration_data.slow_iteration < 5 {
            self.calibration_data.slow_iteration += 1;
        } else {
            self.calibration_data.slow_iteration = 0;
            self.angle_setpoint = if self.angle_setpoint < 359 {
                self.angle_setpoint + 1
            } else {
                self.calibration_data.current_step += 1;
                0
            }
        }
        // Save angle at certain steps
        unsafe {
            if self.calibration_data.current_step < 100 {
                if self.angle_setpoint % 90 == 0 {
                    DEBUG_CALIBRATION_DATA.position_at_cal_step
                        [self.calibration_data.current_step as usize]
                        [(self.angle_setpoint / 90) as usize] = self.get_current_position();
                }
            }
        }

        // Are we done?
        if self.calibration_data.current_step == 100 {
            self.calibration_data.calibrated = true;
            self.mode = Mode::Normal;
        }
    }

    pub fn start_calibration(&mut self) {
        // Reset
        self.angle_setpoint = 0;
        self.position_input.reset();
        self.calibration_data.reset();

        self.mode = Mode::Calibration;
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
    pub fn get_calibration_data(&self) -> &Debug_calibration_data {
        unsafe { &DEBUG_CALIBRATION_DATA }
    }
}
