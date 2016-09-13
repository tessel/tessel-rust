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
    // Acquire port A.
    let (mut port_a, _) = Tessel::ports().unwrap();

    // Create the accelerometer object and connect to the sensor.
    let mut climate = Climate::new(port_a);
    climate.connect().expect("Could not connect to climate sensor.");

    println!("Reading climate sensor... (Press CTRL + C to stop)");
    loop {
        println!("Temperature (Fahrenheit): {:?}", climate.read_temperature());

        // Continue the loop after sleeping for 100ms.
        sleep(Duration::from_millis(100));
    }
}
