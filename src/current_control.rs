use crate::pid::{Controller, PIDController};
use crate::util;

pub trait CurrentOutput {
    fn set_output_value(&mut self, value: i32);
    fn enable(&mut self, enable: bool);
}

pub trait CurrentDevice {
    fn update(&mut self, dt: u32);
    fn set_current(&mut self, milli_amps: i32);
    fn current(&self) -> i32;
    fn enable(&mut self, enable: bool);
    fn force_duty(&mut self, duty: i32);
}

pub trait PIDControl {
    fn set_controller_p(&mut self, value: i32);
    fn set_controller_i(&mut self, value: i32);
    fn set_controller_d(&mut self, value: i32);
}

const ADC_BUFFER_SIZE: usize = 5;
const PID_SCALING_FACTOR: i32 = 100_000;
const PID_DT_SCALE_FACTOR: u32 = 1000;
const MAX_DUTY_CYCLE: i32 = 2500;

/// For now hard bound to ADC1
pub struct CurrentControl<T: CurrentOutput> {
    shunt_resistance: u32,
    current_setpoint: i32,
    adc_value: u32,
    voltage: i32,
    current: i32,
    output: T,
    output_value: i32,
    pid: PIDController<i32>,
    adc_buffer: [u32; ADC_BUFFER_SIZE],
    adc_buffer_index: usize,
    adc_max_value: u32,
    no_pid_control: bool,
}

impl<T: CurrentOutput> CurrentControl<T> {
    pub fn new(shunt_resistance: u32, output: T, adc_max_value: u32) -> Self {
        let mut s = Self {
            shunt_resistance,
            current_setpoint: 0,
            adc_value: 0,
            voltage: 0,
            current: 0,
            output,
            output_value: 0,
            pid: PIDController::new(0, 0, 0), // PID

            adc_buffer: [0; ADC_BUFFER_SIZE],
            adc_buffer_index: 0,
            adc_max_value,
            no_pid_control: false,
        };
        s.pid.set_limits(
            -MAX_DUTY_CYCLE * PID_SCALING_FACTOR,
            MAX_DUTY_CYCLE * PID_SCALING_FACTOR,
        );
        s
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

    pub fn add_sample(&mut self, adc_value: u32) {
        self.adc_buffer[self.adc_buffer_index] = adc_value;
        if self.adc_buffer_index < (ADC_BUFFER_SIZE - 1) {
            self.adc_buffer_index += 1;
        } else {
            self.adc_buffer_index = 0;
        }
    }

    fn calc_voltage(&mut self) {
        self.voltage = ((3300 * self.adc_value) / self.adc_max_value) as i32;
    }

    fn calc_current(&mut self) {
        self.current = if self.voltage >= 0 {
            let current_raw = (self.voltage * 1000) / self.shunt_resistance as i32; // uV / mOhm = mA
            if self.output_value >= 0 {
                current_raw
            } else {
                current_raw * -1
            }
        } else {
            0
        }
    }

    fn average_adc_value(&mut self) {
        self.adc_value = self.adc_buffer.iter().sum::<u32>() / ADC_BUFFER_SIZE as u32;
    }

    fn calc_output(&mut self, dt: u32) {
        if !self.no_pid_control {
            self.output_value = self.pid.update(
                self.current * PID_SCALING_FACTOR as i32,
                (dt / PID_DT_SCALE_FACTOR) as i32,
            ) / PID_SCALING_FACTOR;

            self.output_value = util::clamp(-MAX_DUTY_CYCLE, MAX_DUTY_CYCLE, self.output_value);
        }
        self.output.set_output_value(self.output_value);
    }
}

impl<T: CurrentOutput> CurrentDevice for CurrentControl<T> {
    fn update(&mut self, dt: u32) {
        self.average_adc_value();
        self.calc_voltage();
        self.calc_current();
        self.calc_output(dt);
    }
    fn set_current(&mut self, milli_amps: i32) {
        self.current_setpoint = milli_amps;
        self.pid.set_target(milli_amps * PID_SCALING_FACTOR);
    }
    fn current(&self) -> i32 {
        if self.output_value >= 0 {
            self.current as i32
        } else {
            -1 * self.current as i32
        }
    }
    fn enable(&mut self, enable: bool) {
        if enable {
            //reset
            self.output_value = 0;
            for data in self.adc_buffer.iter_mut() {
                *data = 0;
            }
            self.pid.reset();
            self.no_pid_control = false;
        }
        self.output.enable(enable);
    }
    fn force_duty(&mut self, duty: i32) {
        self.no_pid_control = true;
        self.output_value = duty;
        self.output.enable(true);
    }
}

impl<T: CurrentOutput> PIDControl for CurrentControl<T> {
    fn set_controller_p(&mut self, value: i32) {
        self.pid.p_gain = value;
    }
    fn set_controller_i(&mut self, value: i32) {
        self.pid.i_gain = value;
    }
    fn set_controller_d(&mut self, value: i32) {
        self.pid.d_gain = value;
    }
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCurrentOutput {
        last_output: i32,
    }
    impl Default for MockCurrentOutput {
        fn default() -> Self {
            Self { last_output: 0 }
        }
    }

    impl CurrentOutput for MockCurrentOutput {
        fn set_output_value(&mut self, value: i32) {
            self.last_output = value;
        }
        fn enable(&mut self, _enable: bool) {
            // Nothing to do
        }
    }

    #[test]
    fn motor_pos_test() {
        let mock_current_ouput = MockCurrentOutput::default();

        let shunt_resistance = 400;
        let target_current_mA = 10;
        let mut currentcontrol = CurrentControl::new(shunt_resistance, mock_current_ouput);
        currentcontrol.set_current(target_current_mA);
        currentcontrol.set_controller_p(10);
        currentcontrol.set_controller_i(0);
        currentcontrol.set_controller_d(0);
        currentcontrol.enable(true);

        //for _ in 0..3 {
        let mV = 1;
        currentcontrol.update(1, 1, mV);
        //}

        let current_mA = mV * 1000 / shunt_resistance as i32;
        let output = (target_current_mA - current_mA) * 10;
        assert_eq!(output, currentcontrol.get_current_output().last_output);
    }
}
