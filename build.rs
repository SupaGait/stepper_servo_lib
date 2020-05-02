use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::f32::consts::PI;

#[path = "src/sine_lookup/mod.rs"] mod sine_lookup;
use sine_lookup::SAMPLE_POINTS;
use sine_lookup::SCALING_FACTOR;

fn main() -> std::io::Result<()> {
    // Open the file and write content.
    let out_path = Path::new("src/sine_lookup/lookup_table.rs");
    let mut lookup_file = File::create(out_path).expect("Unable to create file for lookup table generation");

    write!(lookup_file, "pub static SIN_LOOKUP_TABLE: [i32; {}] = [\n", SAMPLE_POINTS)?;
    for point in 0..SAMPLE_POINTS {
        let value = point as f32 / SAMPLE_POINTS as f32;
        let value = (value* 2.0 * PI).sin();
        let value = (value * SCALING_FACTOR as f32) as i32;
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