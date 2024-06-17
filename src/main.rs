use chrono::{DateTime, FixedOffset, Utc};
use esp_idf_hal::{
    delay::{FreeRtos, BLOCK},
    i2c::*,
    peripherals::Peripherals,
    prelude::*,
};
use esp_idf_sys as _;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::time::SystemTime;

mod wifi;
use wifi::*;

mod display;
use display::*;

mod steps;
use steps::*;

const QUERY_NTP_INTERVAL_N_LOOPS: u32 = 36000; // 30 hours (ie. 36000 loops of 3 sec); btw time drift is really really low in my board

fn main() {
    // ESP boilerplate
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Starting clockio, hold on tight");

    let mut peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Set up the I2C interface and the display
    // The way the ESP32 IDF is typed make really hard to move this to specialized functions
    let display_i2c_config = I2cConfig::default().baudrate(esp_idf_hal::prelude::Hertz(100_000)); // 100 kHz
    let display_i2c = I2cDriver::new(
        peripherals.i2c0,
        pins.gpio18,
        pins.gpio19,
        &display_i2c_config,
    )
    .unwrap();
    let interface = I2CDisplayInterface::new(display_i2c);

    // Initialize the display
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let _ = display.init();
    let _ = display.set_brightness(Brightness::DIMMEST);

    // Setup I2C interface for the accelerometer
    let accel_i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let mut accel = I2cDriver::new(
        peripherals.i2c1,
        pins.gpio23,
        pins.gpio22,
        &accel_i2c_config,
    )
    .unwrap();

    // Init accelerometer (MPU6050 chip)
    accel.write(0x68, &[0x6B_u8], BLOCK).unwrap();
    accel.write(0x68, &[0_u8], BLOCK).unwrap();

    // Show a pretty cat face while we get the time
    show_welcome(&mut display);

    connect_wifi_and_update_system_time(&mut peripherals.modem);

    let mut update_counter = 0;
    loop {
        // Show the current time

        let tz_utc_minus_3 = FixedOffset::west_opt(3 * 3600).unwrap();
        let dt_now_utc: DateTime<Utc> = SystemTime::now().clone().into();
        let dt_utc_minus_3 = dt_now_utc.with_timezone(&tz_utc_minus_3);

        let formatted_hour = format!("{}", dt_utc_minus_3.format("%H:%M:%S"));
        let formatted_hour_short = format!("{}", dt_utc_minus_3.format("%H:%M"));

        log::info!("Time is {}", formatted_hour);
        if update_counter % 3 == 0 {
            display_update_with_text(&mut display, &formatted_hour_short);
        }

        if update_counter > QUERY_NTP_INTERVAL_N_LOOPS {
            log::info!(
                "Updating system time via Wifi and NTP because {} intervals have passed...",
                QUERY_NTP_INTERVAL_N_LOOPS
            );
            connect_wifi_and_update_system_time(&mut peripherals.modem);
            update_counter = 0;
        }

        // Now, lets collect accelerometer samples for 3 sec, detect steps and show

        let mut accel_data: [u8; 2] = [0_u8; 2];
        let mut x_serie: [i16; 30] = [0_i16; 30];

        for i in 0..30 {
            accel.write(0x68, &[0x3B_u8], BLOCK).unwrap();
            accel.read(0x68, &mut accel_data, BLOCK).unwrap();

            let x_axis_accel = nice_integer_from_raw_readings(accel_data[0], accel_data[1]);
            x_serie[i] = x_axis_accel;

            FreeRtos::delay_ms(100); // this time is important, don't change without analysis
        }

        detect_steps(x_serie);
        let stepcount = STEPS_COUNT.lock().unwrap();
        let formatted_steps = format!("{}", stepcount);
        log::info!("STEPCOUNT is {}", formatted_steps);
        if update_counter % 3 != 0 {
            display_update_with_text(&mut display, &formatted_steps);
        }

        update_counter += 1;
    }
}
