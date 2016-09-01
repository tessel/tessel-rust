use std::io;
use std::io::prelude::*;
use unix_socket::UnixStream;

/// Starting byte of transmission commands. Because this is extensible, we use
/// a list of constants instead of an enum.
pub mod command {
    pub struct Command(pub u8);

    pub const NOP: Command = Command(0x00);
    pub const FLUSH: Command = Command(0x01);
    pub const ECHO: Command = Command(0x02);
    pub const GPIO_IN: Command = Command(0x03);
    pub const GPIO_HIGH: Command = Command(0x04);
    pub const GPIO_LOW: Command = Command(0x05);
    pub const GPIO_CFG: Command = Command(0x06);
    pub const GPIO_WAIT: Command = Command(0x07);
    pub const GPIO_INT: Command = Command(0x08);
    pub const ENABLE_SPI: Command = Command(0x0A);
    pub const DISABLE_SPI: Command = Command(0x0B);
    pub const ENABLE_I2C: Command = Command(0x0C);
    pub const DISABLE_I2C: Command = Command(0x0D);
    pub const ENABLE_UART: Command = Command(0x0E);
    pub const DISABLE_UART: Command = Command(0x0F);
    pub const TX: Command = Command(0x10);
    pub const RX: Command = Command(0x11);
    pub const TXRX: Command = Command(0x12);
    pub const START: Command = Command(0x13);
    pub const STOP: Command = Command(0x14);
    pub const GPIO_TOGGLE: Command = Command(0x15);
    pub const GPIO_INPUT: Command = Command(0x16);
    pub const GPIO_RAW_READ: Command = Command(0x17);
    pub const ANALOG_READ: Command = Command(0x18);
    pub const ANALOG_WRITE: Command = Command(0x19);
    pub const GPIO_PULL: Command = Command(0x1A);
    pub const PWM_DUTY_CYCLE: Command = Command(0x1B);
    pub const PWM_PERIOD: Command = Command(0x1C);
}

/// Starting byte of reply packets. Because this is extensible, we use
/// a list of constants instead of an enum.
pub mod reply {
    pub struct Reply(pub u8);

    pub const ACK: Reply = Reply(0x80);
    pub const NACK: Reply = Reply(0x81);
    pub const HIGH: Reply = Reply(0x82);
    pub const LOW: Reply = Reply(0x83);
    pub const DATA: Reply = Reply(0x84);

    pub const MIN_ASYNC: Reply = Reply(0xA0);
    /// c0 to c8 is all async pin assignments.
    pub const ASYNC_PIN_CHANGE_N: Reply = Reply(0xC0);
    pub const ASYNC_UART_RX: Reply = Reply(0xD0);
}

/// Socket that communicates with the SAMD21.
pub struct PortSocket {
    socket_path: String,
    socket: UnixStream,
}

impl PortSocket {
    pub fn new(path: &str) -> PortSocket {
        // Connect to the unix domain socket for this port
        let socket = UnixStream::connect(path).unwrap();

        PortSocket {
            socket_path: path.to_string(),
            socket: socket
        }
    }

    pub fn write(&mut self, buffer: &[u8]) -> io::Result<()> {
        try!(self.socket.write(buffer));
        Ok(())
    }

    pub fn write_command(&mut self, cmd: command::Command, buffer: &[u8]) -> io::Result<()> {
        try!(self.socket.write(&[cmd.0]));
        try!(self.socket.write(buffer));
        Ok(())
    }

    pub fn read_exact(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        try!(self.socket.read_exact(buffer));
        Ok(())
    }
}
