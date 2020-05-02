use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::f32::consts::PI;

const SAMPLE_POINTS: u32 = 360 * 4;
const SCALING_FACTOR: f32 = std::u16::MAX as f32;

fn main() -> std::io::Result<()> {
    let out_path = Path::new("src/sine_lookup/lookup_table.rs");
    let mut lookup_file = File::create(out_path).expect("Unable to create file for lookup table generation");

    write!(lookup_file, "static SIN_LOOKUP_TABLE: [u32; {}] = [\n", SAMPLE_POINTS)?;
    for point in 0..SAMPLE_POINTS {
        let value = point as f32 / SAMPLE_POINTS as f32;
        let value = (value* 2.0 * PI).sin() + 1.0;
        let value = (value * SCALING_FACTOR) as u32;
        write!(lookup_file, "{}",value)?;

        if point != SAMPLE_POINTS-1 {
            write!(lookup_file, ",")?;
        }

        if point != 0 && point % 20 == 0 {
            write!(lookup_file, "\n")?;
        }
    }
    write!(lookup_file, "];")?;

    Ok(())
}