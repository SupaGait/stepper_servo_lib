use crate::motor_control::CurrentDevice;
use crate::pid::{Controller, PIDController};
use crate::util;

pub trait CurrentOutput {
    fn set_output_value(&mut self, value: i32);
    fn enable(&mut self, enable: bool);
}

const CURRENT_BUFFER_SIZE: usize = 5;
const PID_SCALING_FACTOR: i32 = 10000;
const PID_DT_SCALE_FACTOR: u32 = 100;

/// For now hard bound to ADC1
pub struct CurrentControl<T: CurrentOutput> {
    shunt_resistance: u32,
    current_setpoint: i32,
    adc_value: u32,
    voltage: i32,
    current: i32,
    output: T,
    //output_value_raw: i32,
    output_value: i32,
    pid: PIDController<i32>,
    current_buffer: [i32; CURRENT_BUFFER_SIZE],
    buffer_index: usize,
}

impl<T: CurrentOutput> CurrentControl<T> {
    pub fn new(shunt_resistance: u32, output: T) -> Self {
        let mut s = Self {
            shunt_resistance,
            current_setpoint: 0,
            adc_value: 0,
            voltage: 0,
            current: 0,
            output,
            //output_value_raw: 0,
            output_value: 0,
            pid: PIDController::new(200, 0, 0), // PID

            current_buffer: [0; CURRENT_BUFFER_SIZE],
            buffer_index: 0,
        };
        s.pid
            .set_limits(-230 * PID_SCALING_FACTOR, 230 * PID_SCALING_FACTOR);
        s
    }

    pub fn set_p_value(&mut self, p_value: i32) {
        self.pid.p_gain = p_value;
    }
    pub fn set_i_value(&mut self, i_value: i32) {
        self.pid.i_gain = i_value;
    }
    pub fn set_d_value(&mut self, d_value: i32) {
        self.pid.d_gain = d_value;
    }

    pub fn adc_value(&self) -> u32 {
        self.adc_value
    }

    pub fn output_value(&self) -> i32 {
        self.output_value
    }

    pub fn voltage(&self) -> i32 {
        self.voltage
    }

    pub fn get_current_output(&mut self) -> &mut T {
        &mut self.output
    }

    pub fn enable(&mut self, enable: bool) {
        if enable {
            //reset
            self.output_value = 0;
            for data in self.current_buffer.iter_mut() {
                *data = 0;
            }
        }
        self.output.enable(enable);
    }

    pub fn update(&mut self, dt: u32, adc_value: u32, adc_voltage: i32) {
        self.adc_value = adc_value;
        self.voltage = adc_voltage;

        let current = if self.voltage > 0 {
            let current_raw = (self.voltage * 1000) / self.shunt_resistance as i32; // uV / mOhm = mA
            if self.output_value > 0 {
                current_raw
            } else {
                current_raw * -1
            }
        } else {
            0
        };

        self.average_current(current, dt / PID_DT_SCALE_FACTOR);
        //self.calc_output(dt);
    }

    fn average_current(&mut self, current: i32, dt: u32) {
        self.current_buffer[self.buffer_index] = current;
        if self.buffer_index < (CURRENT_BUFFER_SIZE - 1) {
            self.buffer_index += 1;
        } else {
            self.buffer_index = 0;
            self.calc_output(dt);
        }
        self.current = self.current_buffer.iter().sum::<i32>() / CURRENT_BUFFER_SIZE as i32;
    }

    fn calc_output(&mut self, dt: u32) {
        self.output_value += self.pid.update(self.current as i32, dt as i32) / PID_SCALING_FACTOR;
        self.output_value = util::clamp(-230, 230, self.output_value);
        self.output.set_output_value(self.output_value);
    }
}

impl<T: CurrentOutput> CurrentDevice for CurrentControl<T> {
    fn set_current(&mut self, milli_amps: i32) {
        self.current_setpoint = milli_amps;
        self.pid.set_target(milli_amps);
    }
    fn current(&self) -> i32 {
        if self.output_value > 0 {
            self.current as i32
        } else {
            -1 * self.current as i32
        }
    }
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motor_pos_test() {
        todo!()
    }
}