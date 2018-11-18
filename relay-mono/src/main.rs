extern crate relay_mono;
extern crate tessel;

use relay_mono::RelayArray;
use std::thread::sleep;
use std::time::Duration;
use tessel::Tessel;

fn main() {
    // Acquire port A.
    let (port_a, _) = Tessel::ports().unwrap();

    // Create the relay array.
    let mut servos = RelayArray::new(port_a);
    servos.connect().expect("Could not connect to relay array.");

    println!("Toggling relays every 1s... (Press CTRL + C to stop)");
    loop {
        println!("[0, 0]");
        sleep(Duration::from_millis(3000));
        servos.set_latch(1, true);
        println!("[1, 0]");
        sleep(Duration::from_millis(3000));
        servos.set_latch(2, true);
        println!("[1, 1]");
        sleep(Duration::from_millis(3000));
        servos.set_latch(1, false);
        println!("[0, 0]");
        sleep(Duration::from_millis(3000));
        servos.set_latch(2, false);
    }
}
