#![cfg_attr(not(test), no_std)]

pub mod motor_control;
pub mod current_control;
pub mod serial_commands;
pub mod util;
pub mod pid;
pub mod coil;
pub mod sine_lookup;