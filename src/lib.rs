use std::fs::File;
use std::io;
use std::io::Write;

// Paths to the SPI daemon sockets with incoming data from coprocessor.
const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

/// Primary exported Tessel object with access to module ports, LEDs, and a button.
/// # Example
/// ```
/// use rust_tessel::Tessel;
///
/// # #[allow(dead_code)]
/// # fn example() {
/// let t = Tessel::new();
/// // Tessel 2 has four LEDs available.
/// assert_eq!(t.led.len(), 4);
/// // Tessel 2 has two ports labelled a and b
/// let a = t.port.a;
/// let b = t.port.b;
/// # }
/// ```
pub struct Tessel {
    // A group of module ports.
    pub port: PortGroup,
    // An array of LED structs.
    pub led: Vec<LED>,
}

impl Tessel {
    // new() returns a Tessel struct conforming to the Tessel 2's functionality.
    pub fn new() -> Tessel {
        // Create a port group with two ports, one on each domain socket path.
        let ports = PortGroup {
            a: Port { socket_path: PORT_A_UDS_PATH },
            b: Port { socket_path: PORT_B_UDS_PATH },
        };

        // Create models for the four LEDs.
        let red_led = LED::new("red", "error");
        let amber_led = LED::new("amber", "wlan");
        let green_led = LED::new("green", "user1");
        let blue_led = LED::new("blue", "user2");

        // Return the Tessel with these fields.
        Tessel {
            port: ports,
            led: vec![red_led, amber_led, green_led, blue_led],
        }
    }
}

/// A PortGroup is a simple way to access each port through its letter identifier.
#[allow(dead_code)]
pub struct PortGroup {
    pub a: Port,
    pub b: Port,
}

/// A Port is a model of the Tessel hardware ports.
/// # Example
/// ```
/// use rust_tessel::Port;
///
/// let p = Port{socket_path: "path/to/my/socket"};
/// ```
pub struct Port {
    // Path of the domain socket.
    pub socket_path: &'static str,
}

// TODO: Figure out how to override the path secretly so the example
// can actually be run.
/// A LED models an LED on the Tessel board.
/// # Example
/// ```rust,no_run
/// use rust_tessel::LED;
///
/// let mut led = LED::new("red", "error");
/// // LEDs are off by default.
/// assert_eq!(false, led.read());
/// led.on().unwrap();
/// assert_eq!(true, led.read());
pub struct LED {
    // The file object we write to in order to change state.
    file: File,
    // The current value of the LED, defaults to false.
    value: bool,
}

impl LED {
    pub fn new(color: &'static str, kind: &'static str) -> LED {
        let path = format!("/sys/devices/leds/leds/tessel:{}:{}/brightness",
                           color,
                           kind);

        // Open the file for write operations.
        LED::new_with_file(File::create(path).unwrap())
    }


    fn new_with_file(file: File) -> LED {
        let mut led = LED {
            value: false,
            file: file,
        };

        // Turn the LED off by default.
        led.off().unwrap();

        led
    }

    // Turn the LED on (same as `high`).
    pub fn on(&mut self) -> Result<(), io::Error> {
        self.high()
    }

    // Turn the LED off (same as `low`).
    pub fn off(&mut self) -> Result<(), io::Error> {
        self.low()
    }

    // Turn the LED on.
    pub fn high(&mut self) -> Result<(), io::Error> {
        self.write(true)
    }

    // Turn the LED off.
    pub fn low(&mut self) -> Result<(), io::Error> {
        self.write(false)
    }

    // Sets the LED to the opposite of its current state.
    pub fn toggle(&mut self) -> Result<(), io::Error> {
        let new_value = !self.value;
        self.write(new_value)
    }

    // Returns the current state of the LED.
    pub fn read(&self) -> bool {
        self.value
    }

    // Helper function to write new state to LED filepath.
    fn write(&mut self, new_value: bool) -> Result<(), io::Error> {
        // Save the new value to the model.
        self.value = new_value;
        // Return the binary representation of that value type.
        let string_value = match new_value {
            true => b'1',
            false => b'0',
        };

        // Write that data to the file and return the result.
        self.file.write_all(&[string_value])
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::*;
    use std::io::{Read, Seek, SeekFrom};

    #[test]
    fn led_writes_to_file() {
        let mut tmpfile = tempfile::tempfile().unwrap();
        // The tmpfile handle can be reused as long as LED gets its own
        // clone of the handle, and we are diligent about seeking.
        // This avoids needing to figure out where the tmpfile is in order
        // to open more handles.
        let mut led = LED::new_with_file(tmpfile.try_clone().unwrap());
        let mut buf = String::new();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        tmpfile.read_to_string(&mut buf).unwrap();
        assert_eq!("0", buf);
        led.on().unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        tmpfile.read_to_string(&mut buf).unwrap();
        // b'1' is written as 001 into the file.
        assert_eq!("001", buf);
    }
}
