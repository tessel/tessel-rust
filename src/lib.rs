use std::fs::File;
use std::io;
use std::io::Write;

// Paths to the SPI daemon sockets with incoming data from coprocessor.
const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

/// Primary exported Tessel object with access to module ports, LEDs, and a button.
/// # Example
/// use rust_tessel::Tessel;
///
/// let t = Tessel::new();
/// // Tessel 2 has four LEDs available.
/// assert_eq(t.led.len(), 4);
/// // Tessel 2 has two ports labelled a and b
/// assert(t.ports.a);
/// assert(t.ports.b);
/// // Tessel 2 has one button.
/// assert(t.button);
pub struct Tessel {
    // A group of module ports.
    pub port: PortGroup,
    // An array of LED structs.
    pub led: Vec<LED>,
    // A single button struct.
    pub button: Button
}

impl Tessel {
    pub fn new() -> Tessel {

        // Create a port group with two ports, one on each domain socket path.
        let ports = PortGroup { a: Port{socket_path: PORT_A_UDS_PATH}, b: Port{socket_path: PORT_B_UDS_PATH}};

        // Create models for the four LEDs.
        let red_led = LED::new("red", "error");
        let amber_led = LED::new("amber", "wlan");
        let green_led = LED::new("green", "user1");
        let blue_led = LED::new("blue", "user2");

        // Create the button.
        let button =  Button{value: false};

        // Return the Tessel with these fields.
        Tessel {
            port: ports,
            led: vec![red_led, amber_led, green_led, blue_led],
            button: button
        }
    }
}

/// A PortGroup is a simple way to access each port through its letter identifier.
#[allow(dead_code)]
pub struct PortGroup {
    a: Port,
    b: Port
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
    pub socket_path: &'static str
}

/// A LED models an LED on the Tessel board.
/// // TODO: Figure out how to override the path so secretly so this can actually be run.
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
    value: bool
}

impl LED {
    pub fn new(color: &'static str, kind: &'static str) -> LED {
        let path = format!("/sys/devices/leds/leds/tessel:{}:{}/brightness", color, kind);

        let mut led = LED {
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

/// A model of the single button on Tessel.
/// # Example
/// ```
/// use rust_tessel::Button;
///
/// let b = Button{value: false};
/// ```
pub struct Button {
    // The button's current state.
    pub value: bool
}

