use crate::coil::Coil;
use crate::current_control::{CurrentDevice, PIDControl};
//use crate::pid::{Controller, PIDController};

pub trait PositionControlled {
    fn set_angle(&mut self, degrees: i32);
    fn get_angle(&self) -> i32;
}

pub trait PositionInput {
    fn get_position(&self) -> i32;
}

pub struct MotorControl<T1, T2, Inp>
where
    T1: CurrentDevice,
    T2: CurrentDevice,
{
    coil_a: Coil<T1>,
    coil_b: Coil<T2>,
    position_input: Inp,
    angle_setpoint: i32,
    position_setpoint: i32,
    speed: u32,
    current: i32,
    rotate_speed: i32,
    control_type: ControlType,
    cycles_in_step: u32,
    //pid: PIDController<i32>,
}

enum ControlType {
    Rotate,
    Position,
    Hold,
}

const DWT_FREQ: u32 = 72_000_000;
const POS_CONTROLLER_PERIOD: u32 = DWT_FREQ / 5000;
const DEGREES_PER_ENCODER_PULSE: i32 = 36;

impl<T1, T2, Inp> MotorControl<T1, T2, Inp>
where
    T1: CurrentDevice + PIDControl,
    T2: CurrentDevice + PIDControl,
    Inp: PositionInput,
{
    pub fn new(output_coil_a: T1, output_coil_b: T2, position_input: Inp) -> Self {
        Self {
            coil_a: Coil::<T1>::new(output_coil_a),
            coil_b: Coil::<T2>::new(output_coil_b),
            position_input,
            angle_setpoint: 0,
            position_setpoint: 0,
            speed: 0,
            current: 0,
            rotate_speed: 10,
            control_type: ControlType::Rotate,
            cycles_in_step: 0,
            //pid: PIDController::new(0, 0, 0),
        }
    }
    // Returns next requested schedule in cycles
    pub fn update(&mut self) -> u32 {
        match self.control_type {
            ControlType::Rotate => {
                static mut DEGREES: i32 = 0;
                unsafe {
                    if self.rotate_speed >= 0 {
                        DEGREES = if DEGREES < 360 { DEGREES + 1 } else { 0 };
                    } else {
                        DEGREES = if DEGREES > 0 { DEGREES - 1 } else { 360 };
                    }

                    self.set_angle(DEGREES);
                };

                // Request next update in..
                let rotate_speed = if self.rotate_speed >= 0 {
                    self.rotate_speed as u32
                } else {
                    (-1 * self.rotate_speed) as u32
                };

                if rotate_speed != 0 {
                    200_000 / rotate_speed
                } else {
                    200_000
                }
            }
            ControlType::Hold => {
                self.coil_a.current_control().set_current(self.current);
                self.coil_b.current_control().set_current(self.current);
                200_000
            }
            ControlType::Position => {
                let position_diff = self.position_setpoint - self.position_input.get_position();
                let delta_cycles = DWT_FREQ / self.speed;

                self.cycles_in_step = if self.cycles_in_step > DWT_FREQ / 360 {
                    if position_diff > 0 {
                        self.set_angle(self.get_angle() + DEGREES_PER_ENCODER_PULSE);
                    }
                    if position_diff < 0 {
                        self.set_angle(self.get_angle() - DEGREES_PER_ENCODER_PULSE);
                    }
                    0
                } else {
                    self.cycles_in_step + delta_cycles
                };
                delta_cycles
            }
        }
    }
    pub fn update_control_loop(&mut self, dt: u32) {
        self.coil_a.current_control().update(dt);
        self.coil_b.current_control().update(dt);
    }
    pub fn coil_a(&mut self) -> &mut Coil<T1> {
        &mut self.coil_a
    }
    pub fn coil_b(&mut self) -> &mut Coil<T2> {
        &mut self.coil_b
    }
    pub fn enable(&mut self, enable: bool) {
        self.coil_a.current_control().enable(enable);
        self.coil_b.current_control().enable(enable);
    }
    pub fn set_current(&mut self, current: i32) {
        self.current = current;
    }
    pub fn set_position(&mut self, position: i32) {
        self.position_setpoint = position;
        self.control_type = ControlType::Position;
    }
    pub fn set_speed(&mut self, speed: i32) {
        self.speed = speed as u32;
    }
    pub fn position_input(&mut self) -> &mut Inp {
        &mut self.position_input
    }
    pub fn rotate(&mut self, speed: i32) {
        self.rotate_speed = speed;
        self.control_type = ControlType::Rotate;
    }
    pub fn hold(&mut self) {
        self.control_type = ControlType::Hold;
    }
    pub fn force_duty(&mut self, duty: i32) {
        self.coil_a.current_control().force_duty(duty);
        self.coil_b.current_control().force_duty(duty);
    }
}

impl<T1, T2, Inp> PositionControlled for MotorControl<T1, T2, Inp>
where
    T1: CurrentDevice,
    T2: CurrentDevice,
{
    fn set_angle(&mut self, degrees: i32) {
        self.angle_setpoint = degrees;
        self.coil_a.set_angle(degrees, self.current);
        self.coil_b.set_angle(degrees + 90, self.current);
    }
    fn get_angle(&self) -> i32 {
        self.angle_setpoint
    }
}

impl<T1, T2, Inp> PIDControl for MotorControl<T1, T2, Inp>
where
    T1: CurrentDevice + PIDControl,
    T2: CurrentDevice + PIDControl,
{
    fn set_controller_p(&mut self, value: i32) {
        self.coil_a.current_control().set_controller_p(value);
        self.coil_b.current_control().set_controller_p(value);
    }
    fn set_controller_i(&mut self, value: i32) {
        self.coil_a.current_control().set_controller_i(value);
        self.coil_b.current_control().set_controller_i(value);
    }
    fn set_controller_d(&mut self, value: i32) {
        self.coil_a.current_control().set_controller_d(value);
        self.coil_b.current_control().set_controller_d(value);
    }
}

//
// Tests
//

// #[cfg(test)]
// mod tests {
//     use super::*;

// }
