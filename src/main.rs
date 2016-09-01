#![feature(alloc_system)]
extern crate alloc_system;

/// A blinky example for Tessel

extern crate tessel;

use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;

pub mod mma84 {
    use tessel;
    use std::io;

    pub struct Accelerometer<'a> {
        i2c: tessel::I2C<'a>,
    }

    impl<'a> Accelerometer<'a> {
        pub fn new<'b>(i2c: tessel::I2C<'b>) -> Accelerometer<'b> {
            Accelerometer {
                i2c: i2c,
            }
        }

        pub fn connect(&mut self) -> io::Result<()> {
            let xt: [u8; 1] = [0x0d];
            let mut xr: [u8; 1] = [0; 1];
            self.i2c.transfer(0x1d, &xt, &mut xr).unwrap();

            if xr[0] == 0x2A {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid connection code."))
            }
        }
    }
}

fn main() {
    // Create a new Tessel
    let mut tessel = Tessel::new();

    let mut acc = mma84::Accelerometer::new(tessel.port.a.i2c(100000).unwrap());
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
    }
}
