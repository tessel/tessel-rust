//! Tessel API and crate.

#[macro_use] extern crate lazy_static;
extern crate atomic_option;
extern crate unix_socket;
extern crate bit_set;

pub mod protocol;
pub mod pin_select;

pub use pin_select::PinSelect;
use atomic_option::AtomicOption;
use protocol::{Command, reply, PortSocket};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use bit_set::BitSet;
use std::sync::{Arc, Mutex, MutexGuard};

// TODO Corking reduces latency, as spid adds overhead for each packet


// Paths to the SPI daemon sockets with incoming data from coprocessor.
const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

const MCU_MAX_SPEED: u32 = 48e6 as u32;
// TODO: Replace with better name
const MCU_MAX_SCL_RISE_TIME_NS: f64 = 1.5e-8 as f64;
const MCU_MAGIC_DIV_FACTOR_FOR_I2C_BAUD: u8 = 2;
const MCU_MAGIC_SUBTRACT_FACTOR_FOR_I2C_BAUD: u8 = 5;

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
/// # }
/// ```
pub struct Tessel {
    // An array of LED structs.
    pub led: Vec<LED>,
}

lazy_static! {
    // Create a tuple with two ports, one on each domain socket path.
    static ref TESSEL_PORTS: AtomicOption<(Port, Port)> = AtomicOption::new(Box::new((
        Port::new(PORT_A_UDS_PATH),
        Port::new(PORT_B_UDS_PATH),
    )));
}

impl Tessel {
    // new() returns a Tessel struct conforming to the Tessel 2's functionality.
    pub fn new() -> Tessel {
        // Create models for the four LEDs.
        let red_led = LED::new("red", "error");
        let amber_led = LED::new("amber", "wlan");
        let green_led = LED::new("green", "user1");
        let blue_led = LED::new("blue", "user2");

        // Return the Tessel with these fields.
        Tessel {
            led: vec![red_led, amber_led, green_led, blue_led],
        }
    }

    pub fn ports() -> Option<(Port, Port)> {
        TESSEL_PORTS.take(Ordering::Relaxed).map(|x| *x)
    }
}

/// A Port is a model of the Tessel hardware ports.
/// # Example
/// ```
/// use tessel::Port;
/// ```
pub struct Port {
    socket: Arc<Mutex<PortSocket>>,
}

impl Port {
    pub fn new(path: &str) -> Port {
        // Create and return the port struct
        Port {
            socket: Arc::new(Mutex::new(PortSocket::new(path))),
        }
    }

    pub fn pins(&mut self) -> (Pin, Pin, Pin) {
        (
            Pin::new(5, self.socket.clone()),
            Pin::new(6, self.socket.clone()),
            Pin::new(7, self.socket.clone()),
        )
    }

    pub fn i2c<'b>(self) -> (I2cPort<'b>, Gpio<'b>) {
        let mut available = BitSet::new();
        for i in 2..8 {
            available.insert(i);
        }
        (I2cPort::new(self.socket.clone()), Gpio::new(self.socket.clone(), available))
    }

    pub fn spi<'b>(self) -> (SpiPort<'b>, Gpio<'b>) {
        let mut available = BitSet::new();
        for i in 0..2 {
            available.insert(i);
        }
        for i in 5..8 {
            available.insert(i);
        }
        (SpiPort::new(self.socket.clone()), Gpio::new(self.socket.clone(), available))
    }
}

/// Gpio is a selection of pins.
#[allow(dead_code)]
pub struct Gpio<'a> {
    socket: Arc<Mutex<PortSocket>>,
    available: BitSet,
    _phantom: PhantomData<&'a Port>,
}

impl<'a> Gpio<'a> {
    pub fn new<'b>(socket: Arc<Mutex<PortSocket>>, available: BitSet) -> Gpio<'b> {
        // Create and return the port struct
        Gpio {
            socket: socket,
            available: available,
            _phantom: PhantomData,
        }
    }

    // TODO return iterator
    //pub fn pins() { }

    pub fn pin_select<H: PinSelect<'a>>(self, select: H) -> H::Output {
        select.select(self.socket.clone())
    }
}

/// A GPIO pin usable as an input or output.
pub struct Pin<'a> {
    index: usize,
    socket: Arc<Mutex<PortSocket>>,
    _phantom: PhantomData<&'a Port>,
}

impl<'a> Pin<'a> {
    fn new<'b>(index: usize, socket: Arc<Mutex<PortSocket>>) -> Pin<'b> {
        Pin {
            index: index,
            socket: socket,
            _phantom: PhantomData,
        }
    }

    pub fn output(&mut self, value: bool) -> io::Result<()> {
        let mut sock = self.socket.lock().unwrap();
        if value {
            sock.write_command(Command::GpioHigh(self.index as u8))
        } else {
            sock.write_command(Command::GpioLow(self.index as u8))
        }
    }

    pub fn high(&mut self) -> io::Result<()> {
        self.output(true)
    }

    pub fn low(&mut self) -> io::Result<()> {
        self.output(false)
    }
}

/// An I2C Port.
pub struct I2cPort<'a> {
    socket: Arc<Mutex<PortSocket>>,
    _phantom: PhantomData<&'a Port>,
}

impl<'p> I2cPort<'p> {
    // TODO: make frequency optional
    fn new<'a>(socket: Arc<Mutex<PortSocket>>) -> I2cPort<'a> {
        let mut i2c = I2cPort {
            socket: socket,
            _phantom: PhantomData,
        };

        // Use 100Khz as default frequency.
        i2c.enable(I2cPort::compute_baud(100_000));

        i2c
    }

    /// Computes the baudrate as used on the Atmel SAMD21 I2C register
    /// to set the frequency of the I2C Clock.
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

    fn enable(&mut self, baud: u8) {
        let mut sock = self.socket.lock().unwrap();
        sock.write_command(Command::EnableI2c { baud: baud }).unwrap();
    }

    fn tx(sock: &mut MutexGuard<PortSocket>, address: u8, write_buf: &[u8]) {
        sock.write_command(Command::Start(address<<1)).unwrap();
        // Write the command and data
        sock.write_command(Command::Tx(write_buf)).unwrap();
    }

    fn rx(sock: &mut MutexGuard<PortSocket>, address: u8, read_buf: &mut [u8]) {
        sock.write_command(Command::Start(address << 1 | 1)).unwrap();
        // Write the command and transfer length
        sock.write_command(Command::Rx(read_buf.len() as u8)).unwrap();
    }

    fn stop(sock: &mut MutexGuard<PortSocket>) {
        // Tell I2C to send STOP condition
        sock.write_command(Command::Stop).unwrap();
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        self.enable(I2cPort::compute_baud(frequency));
    }

    pub fn send(&mut self, address: u8, write_buf: &[u8]) {
        let mut sock = self.socket.lock().unwrap();
        I2cPort::tx(&mut sock, address, write_buf);
        I2cPort::stop(&mut sock);
    }

    pub fn read(&mut self, address: u8, read_buf: &mut [u8]) -> io::Result<()> {
        let mut sock = self.socket.lock().unwrap();
        I2cPort::rx(&mut sock, address, read_buf);
        I2cPort::stop(&mut sock);

        // TODO: this is not how async reads should be handled.
        // Read in first byte.
        let mut read_byte = [0];
        try!(sock.read_exact(&mut read_byte));
        assert_eq!(read_byte[0], reply::DATA.0);
        // Read in data from the socket
        return sock.read_exact(read_buf);
    }

    pub fn transfer(&mut self, address: u8, write_buf: &[u8], read_buf: &mut [u8]) -> io::Result<()> {
        let mut sock = self.socket.lock().unwrap();
        I2cPort::tx(&mut sock, address, write_buf);
        I2cPort::rx(&mut sock, address, read_buf);
        I2cPort::stop(&mut sock);

        // TODO: this is not how async reads should be handled.
        // Read in first byte.
        let mut read_byte = [0];
        try!(sock.read_exact(&mut read_byte));
        assert_eq!(read_byte[0], reply::DATA.0);
        // Read in data from the socket
        return sock.read_exact(read_buf);
    }
}


/// A SPI Port.
pub struct SpiPort<'a> {
    socket: Arc<Mutex<PortSocket>>,
    _phantom: PhantomData<&'a Port>,
}

impl<'p> SpiPort<'p> {
    // TODO: make frequency optional
    fn new<'a>(socket: Arc<Mutex<PortSocket>>) -> SpiPort<'a> {
        let mut spi = SpiPort {
            socket: socket,
            _phantom: PhantomData,
        };

        // Use 1Mhz as default frequency.
        let (reg, div) = SpiPort::compute_baud(1_000_000);
        spi.enable(0, reg, div);

        spi
    }

    /// Computes the baudrate as used on the Atmel SAMD21 SPI register
    /// to set the frequency of the SPI Clock. Returns (register, divisor)
    fn compute_baud(frequency: u32) -> (u8, u8) {
        // spi baud rate is set by the following equation:
        //  f_baud = f_ref/(2*(baud_reg+1))

        // Max baud rate is 24MHz for the SAMD21, min baud rate is 93750 without a clock divisor.
        // With a max clock divisor of 255, slowest clock is 368Hz.
        // (If we switch from 48MHz xtal to 32KHz xtal, this assumption changes.)
        if frequency < 368 || frequency > 24_000_000 {
            panic!("SPI clock must be between 368Hz and 24MHz");
        }

        let reg: u32 = (48_000_000 / (2 * frequency)) - 1;

        // if speed is slower than 93750 then we need a clock divisor.
        // Find the smallest clock divider such that reg is <= 255.
        if reg > 255 {
            // find the clock divider, make sure its at least 1
            let mut div = 48_000_000 / (frequency * (2 * 255 + 2));
            if div == 0 {
                div = 1;
            }

            // if the speed is still too low, set the clock divider to max and set baud accordingly
            if div > 255 {
                let mut reg = frequency / 255;
                if reg == 0 {
                    reg = 1;
                }
                (reg as u8, 255)
            } else {
                // if we can set a clock divider < 255, max out register.
                (reg as u8, div as u8)
            }
        } else {
            // divider == 1
            (reg as u8, 1)
        }
    }

    fn enable(&mut self, mode: u8, reg: u8, div: u8) {
        let mut sock = self.socket.lock().unwrap();
        sock.write_command(Command::EnableSpi {
            mode: mode,
            freq: reg,
            div: div,
        }).unwrap();
    }

    fn tx(sock: &mut MutexGuard<PortSocket>, write_buf: &[u8]) {
        // Write the command and data
        sock.write_command(Command::Tx(write_buf)).unwrap();
    }

    fn rx(sock: &mut MutexGuard<PortSocket>, read_buf: &mut [u8]) {
        // Write the command and transfer length
        sock.write_command(Command::Rx(read_buf.len() as u8)).unwrap();
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        // Use 1Mhz as default frequency.
        let (reg, div) = SpiPort::compute_baud(frequency);
        self.enable(0, reg, div);
    }

    pub fn send(&mut self, write_buf: &[u8]) {
        let mut sock = self.socket.lock().unwrap();
        SpiPort::tx(&mut sock, write_buf);
    }

    pub fn read(&mut self, read_buf: &mut [u8]) -> io::Result<()> {
        let mut sock = self.socket.lock().unwrap();
        SpiPort::rx(&mut sock, read_buf);

        // TODO: this is not how async reads should be handled.
        // Read in first byte.
        let mut read_byte = [0];
        try!(sock.read_exact(&mut read_byte));
        assert_eq!(read_byte[0], reply::DATA.0);
        // Read in data from the socket
        return sock.read_exact(read_buf);
    }

    pub fn transfer(&mut self, write_buf: &[u8], read_buf: &mut [u8]) -> io::Result<()> {
        let mut sock = self.socket.lock().unwrap();
        SpiPort::tx(&mut sock, write_buf);
        SpiPort::rx(&mut sock, read_buf);

        // TODO: this is not how async reads should be handled.
        // Read in first byte.
        let mut read_byte = [0];
        try!(sock.read_exact(&mut read_byte));
        assert_eq!(read_byte[0], reply::DATA.0);
        // Read in data from the socket
        return sock.read_exact(read_buf);
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
