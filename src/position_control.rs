pub enum Direction {
    Increased(i32),
    Decreased(i32),
    Unknown(i32),
}
pub trait PositionInput {
    fn get_position(&self) -> i32;
    fn get_direction(&self) -> Direction;
}
pub struct PositionControl<Input>{
    position_input: Input,
    position: i32,
    setpoint: i32,
    speed: i32,
    detected_angle: i32,
    requested_angle: i32,
}

const DWT_FREQ: i32 = 72_000_000;
const UPDATE_PERIOD: i32 = DWT_FREQ / 10_000;
const DEGREES_PER_ENCODER_PULSE: i32 = 36;

impl<Input> PositionControl<Input>
where
    Input: PositionInput,
{
    pub fn new(position_input: Input) -> Self {
        Self {
            position_input,
            position: 0,
            setpoint: 0,
            speed: 0,
            detected_angle: 0,
            requested_angle: 0,
        }
    }
    pub fn update_position(&mut self, direction: Direction) {
        match direction {
            Direction::Increased(value) => {
                self.detected_angle = self.detected_angle + DEGREES_PER_ENCODER_PULSE;
                self.position = value;
            },
            Direction::Decreased(value) => {
                self.detected_angle = self.detected_angle - DEGREES_PER_ENCODER_PULSE;
                self.position = value;
            },
            _ => (),
        }
    }
    pub fn update(&mut self) -> u32 {
        let position_diff = self.setpoint - self.position;


        UPDATE_PERIOD as u32
    }

    pub fn angle(&self) -> i32 {
        self.detected_angle
    }
    pub fn set_position(&mut self, position: i32) {
        self.setpoint = position;
    }
    pub fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
    }
}