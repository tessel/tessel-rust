use std::fs::File;
use std::io;
use std::io::Write;

// Paths to the SPI daemon sockets with incoming data from coprocessor.
const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

// Primary exported Tessel object with access to module ports, LEDs, and a button.
#[allow(dead_code)]
pub struct Tessel {
    // A group of module ports.
    pub port: PortGroup,
    // An array of LED structs.
    pub led: Vec<LED>,
    // A single button struct.
    pub button: Button
}

impl Tessel {
    // Returns a new Tessel object.
    pub fn new() -> Tessel {

        // Create a port group with two ports, one on each domain socket path.
        let ports = PortGroup { a: Port::new(PORT_A_UDS_PATH), b: Port::new(PORT_B_UDS_PATH) };

        // Create models for the four LEDs.
        let red_led = LED::new("red", "error");
        let amber_led = LED::new("amber", "wlan");
        let green_led = LED::new("green", "user1");
        let blue_led = LED::new("blue", "user2");

        // Create the button.
        let button =  Button::new();

        // Return the Tessel with these fields.
        Tessel {
            port: ports,
            led: vec![red_led, amber_led, green_led, blue_led],
            button: button
        }
    }
}

// A PortGroup is a simple way to access each port through its letter identifier.
#[allow(dead_code)]
pub struct PortGroup {
    a: Port,
    b: Port
}

// A Port is a model of the Tessel hardware ports.
#[allow(dead_code)]
pub struct Port {
    // Path of the domain socket.
    socket_path: &'static str
}

impl Port {
    pub fn new(path: &'static str) -> Port {
        Port {
            socket_path: path
        }
    }
}

#[allow(dead_code)]
pub struct LED {
    // The color of the given LED (used in filepath creation).
    color: &'static str,
    // The type of LED (used in filepath creation).
    kind: &'static str,
    // The file object we write to in order to change state.
    file: File,
    // The current value of the LED, defaults to false.
    value: bool
}

impl LED {
    pub fn new(color: &'static str, kind: &'static str) -> LED {

        let path = format!("/sys/devices/leds/leds/tessel:{}:{}/brightness", color, kind);

        let mut led = LED {
            color: color,
            kind: kind,
            value: false,
            // Open the file for write operations.
            file: File::create(path).unwrap()
        };

        // Turn the LED off by default.
        led.off().unwrap();

        led
    }

    // Turn the LED on (same as `high`).
    pub fn on(&mut self)-> Result<(), io::Error> {
        self.high()
    }

    // Turn the LED off (same as `low`).
    pub fn off(&mut self)-> Result<(), io::Error> {
        self.low()
    }

    // Turn the LED on.
    pub fn high(&mut self)-> Result<(), io::Error> {
        self.write(true)
    }

    // Turn the LED off.
    pub fn low(&mut self)-> Result<(), io::Error> {
        self.write(false)
    }

    // Sets the LED to the opposite of its current state.
    pub fn toggle(&mut self)-> Result<(), io::Error> {
        let new_value = !self.value;
        self.write(new_value)
    }

    // Returns the current state of the LED.
    pub fn read(&self)-> bool {
        self.value
    }

    // Helper function to write new state to LED filepath.
    fn write(&mut self, new_value: bool)-> Result<(), io::Error> {
        // Save the new value to the model.
        self.value = new_value;
        // Return the binary representation of that value type.
        let string_value = match new_value {
            true => b'1',
            false => b'0'
        };

        // Write that data to the file and return the result.
        self.file.write_all(&[string_value])
    }
}

// A model of the single button on Tessel.
#[allow(dead_code)]
pub struct Button {
    // The button's current state.
    value: bool
}

impl Button {
    pub fn new() -> Button {
        Button {
            value: false
        }
    }
}
