#![feature(alloc_system)]
extern crate alloc_system;

extern crate accel_mma84;
extern crate tessel;

use accel_mma84::Accelerometer;
use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;

fn main() {
    // Create a new Tessel
    let mut tessel = Tessel::new();

    let mut acc = Accelerometer::new(tessel.port.a.i2c(100000).unwrap());
    acc.connect().expect("Could not connect to accelerometer.");
    println!("Connected!");

    // Turn on one of the LEDs
    tessel.led[2].on().unwrap();

    println!("I'm blinking! (Press CTRL + C to stop)");

    // Loop forever
    loop {
        // Toggle each LED
        tessel.led[2].toggle().unwrap();
        tessel.led[3].toggle().unwrap();
        // Re-execute the loop after sleeping for 100ms
        sleep(Duration::from_millis(100));

        println!("acceleration: {:?}", acc.read_acceleration());

        let _ = std::io::stdout().flush();
    }
}
