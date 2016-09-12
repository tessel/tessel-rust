#![feature(alloc_system)]
extern crate alloc_system;

extern crate climate_si7020;
extern crate tessel;

use climate_si7020::Climate;
use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;

fn main() {
    // Create a new Tessel
    let mut tessel = Tessel::new();

    println!("Starting.");

    let (port_a, _) = Tessel::ports().unwrap();
    let mut climate = Climate::new(port_a);
    climate.connect().expect("Could not connect to climate sensor.");
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

        println!("temperature: {:?}", climate.read_temperature());

        let _ = std::io::stdout().flush();
    }
}
