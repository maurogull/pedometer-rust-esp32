use std::sync::Mutex;

pub static STEPS_COUNT: Mutex<u32> = Mutex::new(0);

pub fn nice_integer_from_raw_readings(high_byte: u8, low_byte: u8) -> i16 {
    let value: u16 = (high_byte as u16) << 8 | (low_byte as u16);

    if value & 0x8000 != 0 {
        // adjust sign to return a signed i16
        return (value as i16) | !0x7FFF;
    } else {
        return value as i16;
    }
}

pub fn detect_steps(mut serie: [i16; 30]) {
    let mut inc_to_dec_changes: Vec<u8> = Vec::new();
    let mut increasing = true;
    let steps: u32;

    // Downscaling to reduce noise
    for i in 0..30 {
        serie[i] /= 20000;
    }

    // Lets find the curve changes
    for i in 1..30 {
        if increasing {
            if serie[i] < serie[i - 1] {
                increasing = false;
                inc_to_dec_changes.push(i as u8)
            }
        } else {
            if serie[i] > serie[i - 1] {
                increasing = true;
            }
        }
    }

    // Consider a "step" to the time between two high peaks, so dividing by 2
    steps = ((inc_to_dec_changes.len() + 1) / 2) as u32;

    if steps > 0 {
        let mut count = STEPS_COUNT.lock().unwrap();
        *count += steps;
        log::info!("Detected new {} steps", count);
    }
}
