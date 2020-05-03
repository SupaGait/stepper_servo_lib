use crate::coil::Coil;
use crate::current_control::{CurrentDevice, PIDControl};

pub trait PositionControlled {
    fn set_angle(&mut self, degrees: i32);
    fn get_angle(&self) -> i32;
}

pub struct MotorControl<T1, T2>
where T1: CurrentDevice,
      T2: CurrentDevice,
{
    coil_a: Coil<T1>,
    coil_b: Coil<T2>,
    angle: i32,
    current: i32,
}

impl<T1, T2> MotorControl<T1, T2> 
where T1: CurrentDevice + PIDControl,
      T2: CurrentDevice + PIDControl,
{
    pub fn new(output_coil_a: T1, output_coil_b: T2) -> Self {
        Self {
            coil_a: Coil::<T1>::new(output_coil_a),
            coil_b: Coil::<T2>::new(output_coil_b),
            angle: 0,
            current: 0,
        }
    }
    pub fn update(&mut self) {
        todo!()
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
}

impl<T1, T2> PositionControlled for MotorControl<T1, T2>
where T1: CurrentDevice,
      T2: CurrentDevice,
{
    fn set_angle(&mut self, degrees: i32) {
        self.angle = degrees;
        self.coil_a.set_angle(degrees, self.current);
        self.coil_b.set_angle(degrees + 90, self.current);
    }
    fn get_angle(&self) -> i32 {
        self.angle
    }
}

impl<T1,T2> PIDControl for MotorControl<T1, T2> 
where T1: CurrentDevice + PIDControl,
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
