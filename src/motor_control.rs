use crate::coil::Coil;
use crate::current_control::{CurrentDevice, PIDControl};
use crate::position_control::{PositionControl, PositionInput};
//use crate::pid::{Controller, PIDController};

const DWT_FREQ: i32 = 72_000_000;
const UPDATE_PERIOD: i32 = DWT_FREQ / 20_000;

pub trait PositionControlled {
    fn set_angle(&mut self, degrees: i32);
    fn get_angle(&self) -> i32;
}

pub struct MotorControl<T1, T2, Inp>
where
    T1: CurrentDevice,
    T2: CurrentDevice,
{
    coil_a: Coil<T1>,
    coil_b: Coil<T2>,
    position_control: PositionControl<Inp>,
    angle_setpoint: i32,
    current: i32,
    rotate_speed: i32,
    control_type: ControlType,
    enabled: bool,
    //pid: PIDController<i32>,
}

enum ControlType {
    Rotate,
    Position,
    Hold,
    Calibration,
}

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
            position_control: PositionControl::<Inp>::new(position_input, UPDATE_PERIOD),
            angle_setpoint: 0,
            current: 0,
            rotate_speed: 10,
            control_type: ControlType::Hold,
            enabled: false,
            //pid: PIDController::new(0, 0, 0),
        }
    }
    // Returns next requested schedule in cycles
    pub fn update(&mut self) -> u32 {
        if !self.enabled {
            return DWT_FREQ as u32 / 100;
        }

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
                self.position_control.update();
                let angle = self.position_control.angle();
                self.set_angle(angle);

                UPDATE_PERIOD as u32
            }
            ControlType::Calibration => {
                self.position_control.calibrate();

                if self.position_control.calibration_is_done() {
                    self.enable(false);
                    self.control_type = ControlType::Hold;
                } else {
                    let angle = self.position_control.angle();
                    self.set_angle(angle);
                }

                UPDATE_PERIOD as u32
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
        self.enabled = enable;
    }
    pub fn set_current(&mut self, current: i32) {
        self.current = current;
    }
    pub fn set_position(&mut self, position: i32) {
        self.position_control.set_position(position);
        self.control_type = ControlType::Position;
    }
    pub fn set_speed(&mut self, speed: i32) {
        self.position_control.set_speed(speed);
        self.control_type = ControlType::Position;
    }
    pub fn position_control(&mut self) -> &mut PositionControl<Inp> {
        &mut self.position_control
    }
    pub fn handle_new_position(&mut self) {
        self.position_control.update_position();
    }
    pub fn rotate(&mut self, speed: i32) {
        self.rotate_speed = speed;
        self.control_type = ControlType::Rotate;
    }
    pub fn hold(&mut self) {
        self.control_type = ControlType::Hold;
    }
    pub fn calibrate(&mut self) {
        self.control_type = ControlType::Calibration;
        self.position_control.start_calibration();
        self.enable(true);
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
