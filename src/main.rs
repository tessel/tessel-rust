/// A blinky example for Tessel
extern crate rust_tessel;

use rust_tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut tessel = Tessel::new();
    let ref mut green_led = tessel.led[2];

    loop {
        green_led.toggle().unwrap();
        sleep(Duration::from_millis(250));
    }
}
