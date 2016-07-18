#![feature(alloc_system)]
extern crate alloc_system;

/// A blinky example for Tessel

// Import the tessel library
extern crate rust_tessel;
// Import the Tessel API
use rust_tessel::Tessel;
// Import sleep from the standard lib
use std::thread::sleep;
// Import durations from the standard lib
use std::time::Duration;

fn main() {
    // Create a new Tessel
    let mut tessel = Tessel::new();

    let mut i2c = tessel.port.a.i2c(0x1d, 100000);
    println!("Created the I2C Port");
    let xt: [u8; 1] = [0x0d];
    let mut xr: [u8; 1] = [0; 1];
    i2c.transfer(&xt, &mut xr).unwrap();
    println!("Trasnferring i2c data complete {}", xr[0]);

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
    }
}
