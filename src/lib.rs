extern crate unix_socket;

use std::fs::File;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::Error;
use std::u8;
use unix_socket::UnixStream;


// Paths to the SPI daemon sockets with incoming data from coprocessor.
const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

const MCU_MAX_SPEED: u32 = 48e6 as u32;
// TODO: Replace with better name
const MCU_MAX_SCL_RISE_TIME_NS: f64 = 1.5e-8 as f64;
const MCU_MAGIC_DIV_FACTOR_FOR_I2C_BAUD: u8 = 2;
const MCU_MAGIC_SUBTRACT_FACTOR_FOR_I2C_BAUD: u8 = 5;

pub mod command {

    pub const NOP: u8 = 0x00;
    pub const FLUSH: u8 = 0x01;
    pub const ECHO: u8 = 0x02;
    pub const GPIO_IN: u8 = 0x03;
    pub const GPIO_HIGH: u8 = 0x04;
    pub const GPIO_LOW: u8 = 0x05;
    pub const GPIO_CFG: u8 = 0x06;
    pub const GPIO_WAIT: u8 = 0x07;
    pub const GPIO_INT: u8 = 0x08;
    pub const ENABLE_SPI: u8 = 0x0A;
    pub const DISABLE_SPI: u8 = 0x0B;
    pub const ENABLE_I2C: u8 = 0x0C;
    pub const DISABLE_I2C: u8 = 0x0D;
    pub const ENABLE_UART: u8 = 0x0E;
    pub const DISABLE_UART: u8 = 0x0F;
    pub const TX: u8 = 0x10;
    pub const RX: u8 = 0x11;
    pub const TXRX: u8 = 0x12;
    pub const START: u8 = 0x13;
    pub const STOP: u8 = 0x14;
    pub const GPIO_TOGGLE: u8 = 0x15;
    pub const GPIO_INPUT: u8 = 0x16;
    pub const GPIO_RAW_READ: u8 = 0x17;
    pub const ANALOG_READ: u8 = 0x18;
    pub const ANALOG_WRITE: u8 = 0x19;
    pub const GPIO_PULL: u8 = 0x1A;
    pub const PWM_DUTY_CYCLE: u8 = 0x1B;
    pub const PWM_PERIOD: u8 = 0x1C;
}

/// Primary exported Tessel object with access to module ports, LEDs, and a button.
/// # Example
/// ```
/// use tessel::Tessel;
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
            a: Port::new(PORT_A_UDS_PATH),
            b: Port::new(PORT_B_UDS_PATH),
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
/// use tessel::Port;
///
/// let p = Port{socket_path: "path/to/my/socket"};
/// ```
pub struct Port {
    // Path of the domain socket.
    pub socket_path: &'static str,
    pub socket: UnixStream,
}

impl Port {
    pub fn new(path: &'static str) -> Port {
        // Connect to the unix domain socket for this port
        let socket = UnixStream::connect(path).unwrap();
        // Create and return the port struct
        Port {
            socket_path: path,
            socket: socket,
        }
    }

    pub fn i2c(&mut self, address: u8, frequency: u32) -> I2C {
        // Create and return the I2C struct
        I2C::new(self, address, frequency)
    }
}

pub struct I2C<'p> {
    pub port: &'p mut Port,
    pub address: u8,
    pub frequency: u32,
}

impl<'p> I2C<'p> {
    // TODO: make frequency optional
    pub fn new(port: &mut Port, address: u8, frequency: u32) -> I2C {

        let baud: u8 = I2C::compute_baud(frequency);

        let i2c = I2C {
            port: port,
            address: address,
            frequency: frequency,
        };

        i2c.port.socket.write(&[command::ENABLE_I2C, baud]).unwrap();

        i2c
    }

    /// Computes the baudrate as used on the Atmel SAMD21 I2C register
    /// to set the frequency of the I2C Clock
    /// # Example
    /// ```
    /// assert_eq!(compute_baud(1000000), 4);
    /// ``
    fn compute_baud(frequency: u32) -> u8 {

        let mut intermediate: f64 = MCU_MAX_SPEED as f64 / frequency as f64;
        intermediate = intermediate - MCU_MAX_SPEED as f64 * MCU_MAX_SCL_RISE_TIME_NS;
        // TODO: Do not hardcode these numbers
        intermediate = intermediate / MCU_MAGIC_DIV_FACTOR_FOR_I2C_BAUD as f64 -
                       MCU_MAGIC_SUBTRACT_FACTOR_FOR_I2C_BAUD as f64;

        // Return either the intermediate value or 255
        let low = intermediate.min(u8::max_value() as f64);

        // If we have a potentially negative register value
        // Casting as i64 because .float does not seem to work
        if (low as i64) < u8::min_value() as i64 {
            // Use 0 instead
            return u8::min_value();
        } else {
            // Return the new register value
            return low as u8;
        }
    }

    pub fn send(&mut self, write_buf: &[u8]) {
        // TODO: Handle case where buf size is larger than u8::max_size()
        self.port.socket.write(&[command::START, self.address << 1]).unwrap();
        // Write the command and transfer length
        self.port.socket.write(&[command::TX, write_buf.len() as u8]).unwrap();
        // Write the buffer itself
        self.port.socket.write(write_buf).unwrap();
        // Tell I2C to send STOP condition
        self.port.socket.write(&[command::STOP]).unwrap();
    }

    pub fn read(&mut self, read_buf: &mut [u8]) -> Result<(), Error> {
        // TODO: Handle case where buf size is larger than u8::max_size()
        self.port.socket.write(&[command::START, self.address << 1 | 1]).unwrap();
        // Write the command and transfer length
        self.port.socket.write(&[command::RX, read_buf.len() as u8]).unwrap();
        // Tell I2C to send STOP condition
        self.port.socket.write(&[command::STOP]).unwrap();
        // Read in data from the socket
        return self.port.socket.read_exact(read_buf);
    }

    pub fn transfer(&mut self, write_buf: &[u8], read_buf: &mut [u8]) -> Result<(), Error> {
        // TODO: Handle case where buf size is larger than u8::max_size()
        self.port.socket.write(&[command::START, self.address << 1 | 1]).unwrap();
        // Write the command and transfer length
        self.port.socket.write(&[command::TX, write_buf.len() as u8]).unwrap();
        // Send start command again for the subsequent read
        self.port.socket.write(&[command::START, self.address << 1 | 1]).unwrap();
        // Write the command and transfer length
        self.port.socket.write(&[command::RX, read_buf.len() as u8]).unwrap();
        // Tell I2C to send STOP condition
        self.port.socket.write(&[command::STOP]).unwrap();
        // Read in data from the socket
        return self.port.socket.read_exact(read_buf);
    }
}

// TODO: Figure out how to override the path secretly so the example
// can actually be run.
/// A LED models an LED on the Tessel board.
/// # Example
/// ```rust,no_run
/// use tessel::LED;
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
