extern crate accel_mma84;
extern crate tessel;

use accel_mma84::Accelerometer;
use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Acquire port A.
    let (port_a, _) = Tessel::ports().unwrap();

    // Create the accelerometer object and connect to the sensor.
    let mut acc = Accelerometer::new(port_a);
    acc.connect().expect("Could not connect to accelerometer.");

    println!("Reading acceleration sensor... (Press CTRL + C to stop)");
    loop {
        println!("Acceleration (x, y, z): {:?}", acc.read_acceleration());

        // Continue the loop after sleeping for 100ms.
        sleep(Duration::from_millis(100));
    }
}
