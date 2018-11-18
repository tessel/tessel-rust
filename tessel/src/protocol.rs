use std::io;
use std::io::prelude::*;
use unix_socket::UnixStream;

use self::Command::*;

mod raw_cmd {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Command<'a> {
    Nop,
    Flush,

    GpioIn(u8),
    GpioHigh(u8),
    GpioLow(u8),
    GpioToggle(u8),
    GpioWait(u8),
    GpioInt(u8),
    GpioCfg(u8),
    GpioInput(u8),
    GpioRawRead(u8),
    GpioPull(u8),
    AnalogRead(u8),

    AnalogWrite {
        pin: u8,
        value: u8,
    },

    EnableSpi {
        mode: u8,
        freq: u8,
        div: u8,
    },
    DisableSpi,
    EnableI2c {
        baud: u8,
    },
    DisableI2c,
    EnableUart {
        baud: u8,
        mode: u8,
    },
    DisableUart,

    Start(u8),
    Stop,

    PwmDutyCycle {
        pin: u8,
        duty_cycle: u16,
    },
    PwmPeriod {
        prescalar: u8,
        tcc_id: u8,
        period: u16,
    },

    Rx(u8),
    Echo(&'a [u8]),
    Tx(&'a [u8]),
    TxRx(&'a [u8]),
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
    _socket_path: String,
    socket: UnixStream,
}

impl PortSocket {
    pub fn new(path: &str) -> PortSocket {
        // Connect to the unix domain socket for this port
        let socket = UnixStream::connect(path).unwrap();

        PortSocket {
            _socket_path: path.to_string(),
            socket: socket,
        }
    }

    pub fn raw_write(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.socket.write_all(buffer)
    }

    pub fn write_command(&mut self, cmd: Command) -> io::Result<()> {
        let socket = &mut self.socket;
        match cmd {
            Nop => socket.write_all(&[raw_cmd::NOP]),
            Flush => socket.write_all(&[raw_cmd::FLUSH]),
            Rx(len) => socket.write_all(&[raw_cmd::RX, len]),
            Echo(data) => {
                assert!(data.len() <= u8::max_value() as usize);
                try!(socket.write_all(&[raw_cmd::ECHO, data.len() as u8]));
                socket.write_all(data)
            }
            Tx(data) => {
                for slice in data.chunks(u8::max_value() as usize) {
                    try!(socket.write_all(&[raw_cmd::TX, slice.len() as u8]));
                    try!(socket.write_all(slice));
                }
                Ok(())
            }
            TxRx(data) => {
                assert!(data.len() <= u8::max_value() as usize);
                try!(socket.write_all(&[raw_cmd::TXRX, data.len() as u8]));
                socket.write_all(data)
            }
            GpioIn(pin) => socket.write_all(&[raw_cmd::GPIO_IN, pin]),
            GpioHigh(pin) => socket.write_all(&[raw_cmd::GPIO_HIGH, pin]),
            GpioLow(pin) => socket.write_all(&[raw_cmd::GPIO_LOW, pin]),
            GpioToggle(pin) => socket.write_all(&[raw_cmd::GPIO_TOGGLE, pin]),
            GpioWait(pin) => socket.write_all(&[raw_cmd::GPIO_WAIT, pin]),
            GpioInt(pin) => socket.write_all(&[raw_cmd::GPIO_INT, pin]),
            GpioCfg(pin) => socket.write_all(&[raw_cmd::GPIO_CFG, pin]),
            GpioInput(pin) => socket.write_all(&[raw_cmd::GPIO_INPUT, pin]),
            GpioRawRead(pin) => socket.write_all(&[raw_cmd::GPIO_RAW_READ, pin]),
            GpioPull(pin) => socket.write_all(&[raw_cmd::GPIO_PULL, pin]),
            AnalogRead(pin) => socket.write_all(&[raw_cmd::ANALOG_READ, pin]),

            AnalogWrite { pin, value } => socket.write_all(&[raw_cmd::ANALOG_WRITE, pin, value]),

            EnableSpi { mode, freq, div } => {
                socket.write_all(&[raw_cmd::ENABLE_SPI, mode, freq, div])
            }
            DisableSpi => socket.write_all(&[raw_cmd::DISABLE_SPI]),
            EnableI2c { baud } => socket.write_all(&[raw_cmd::ENABLE_I2C, baud]),
            DisableI2c => socket.write_all(&[raw_cmd::DISABLE_I2C]),
            EnableUart { baud, mode } => socket.write_all(&[raw_cmd::ENABLE_UART, baud, mode]),
            DisableUart => socket.write_all(&[raw_cmd::DISABLE_UART]),

            Start(addr) => socket.write_all(&[raw_cmd::START, addr]),
            Stop => socket.write_all(&[raw_cmd::STOP]),

            PwmDutyCycle { pin, duty_cycle } => socket.write_all(&[
                raw_cmd::PWM_DUTY_CYCLE,
                pin,
                (duty_cycle >> 8) as u8,
                (duty_cycle & 0xFF) as u8,
            ]),
            PwmPeriod {
                prescalar,
                tcc_id,
                period,
            } => socket.write_all(&[
                raw_cmd::PWM_PERIOD,
                prescalar << 4 | tcc_id & 0x7,
                (period >> 8) as u8,
                (period & 0xf) as u8,
            ]),
        }
    }

    pub fn read_exact(&mut self, buffer: &mut [u8]) -> io::Result<()> {
        self.socket.read_exact(buffer)
    }
}
