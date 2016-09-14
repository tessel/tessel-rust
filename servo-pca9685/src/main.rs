#![feature(alloc_system)]
extern crate alloc_system;

extern crate servo_pca9685;
extern crate tessel;

use servo_pca9685::ServoArray;
use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Acquire port A.
    let (port_a, _) = Tessel::ports().unwrap();

    // Create the accelerometer object and connect to the sensor.
    let mut servos = ServoArray::new(port_a, false, false);
    servos.connect().expect("Could not connect to servo array.");

    println!("Turning servos... (Press CTRL + C to stop)");
    loop {
        //println!("Temperature (Fahrenheit): {:?}", climate.read_temperature());

        // Continue the loop after sleeping for 100ms.
        println!("1");
        servos.set_duty_cycle(1, 1.0);
        sleep(Duration::from_millis(1000));
        println!("2");
        servos.set_duty_cycle(1, 0.0);
        sleep(Duration::from_millis(1000));
    }
}
