#![feature(alloc_system)]

extern crate alloc_system;
extern crate tessel;

use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;

pub mod mma84 {
    use tessel;
    use std::io;

    #[repr(u8)]
    pub enum ScaleRange {
        Scale2G = 0x00,
        Scale4G = 0x01,
        Scale8G = 0x10,
    }

    #[repr(u8)]
    #[allow(dead_code)]
    enum Command {
        OutXMsb = 0x01,
        XyzDataCfg = 0x0E,
        WhoAmI = 0x0D,
        CtrlReg1 = 0x2A,
        CtrlReg4 = 0x2D,
    }

    pub struct Accelerometer<'a> {
        i2c: tessel::I2C<'a>,
    }

    #[repr(u8)]
    #[allow(non_camel_case_types)]
    pub enum OutputRate {
        Rate800 = 0,
        Rate400 = 1,
        Rate200 = 2,
        Rate100 = 3,
        Rate50 = 4,
        Rate12_5 = 5,
        Rate6_25 = 6,
        Rate1_56 = 7,
    }

    const I2C_ID: u8 = 0x1d;

    impl<'a> Accelerometer<'a> {
        pub fn new<'b>(i2c: tessel::I2C<'b>) -> Accelerometer<'b> {
            Accelerometer {
                i2c: i2c,
            }
        }

        fn read_register(&mut self, cmd: Command) -> io::Result<u8> {
            let mut xr: [u8; 1] = [0; 1];
            try!(self.read_registers(cmd, &mut xr));
            Ok(xr[0])
        }

        /// Reads sequential buffers.
        fn read_registers(&mut self, cmd: Command, buf: &mut [u8]) -> io::Result<()> {
            try!(self.i2c.transfer(I2C_ID, &[cmd as u8], buf));
            Ok(())
        }

        fn write_register(&mut self, cmd: Command, value: u8) -> io::Result<()> {
            self.i2c.send(I2C_ID, &[cmd as u8, value]);
            Ok(())
        }

        pub fn connect(&mut self) -> io::Result<()> {
            println!("verify");
            if try!(self.read_register(Command::WhoAmI)) != 0x2A {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid connection code."))
            }

            println!("but");
            try!(self.set_scale_range(ScaleRange::Scale2G));
            println!("trust");
            try!(self.set_output_rate(OutputRate::Rate1_56));
            println!("ok.");

            Ok(())
        }

        fn standby_enable(&mut self) -> io::Result<()> {
            // Sets the MMA8452 to standby mode.
            let value = try!(self.read_register(Command::CtrlReg1));
            self.write_register(Command::CtrlReg1, value & !(0x01u8))
        }

        fn standby_disable(&mut self) -> io::Result<()> {
            // Sets the MMA8452 to active mode.
            let value = try!(self.read_register(Command::CtrlReg1));
            self.write_register(Command::CtrlReg1, value | (0x01u8))
        }

        pub fn set_scale_range(&mut self, range: ScaleRange) -> io::Result<()> {
            try!(self.standby_enable());
            try!(self.write_register(Command::XyzDataCfg, range as u8));
            try!(self.standby_disable());

            Ok(())
        }

        pub fn set_output_rate(&mut self, rate: OutputRate) -> io::Result<()> {
            try!(self.standby_enable());

            // Clear the three bits of output rate control (0b11000111 = 199)
            let mut value = try!(self.read_register(Command::CtrlReg1));
            value &= 0b11000111;
            try!(self.write_register(Command::CtrlReg1, value | ((rate as u8) << 3)));

            try!(self.standby_disable());

            Ok(())
        }

        pub fn read_acceleration(&mut self) -> io::Result<(f64, f64, f64)> {
            let mut buf = [0; 6];
            try!(self.read_registers(Command::OutXMsb, &mut buf));

            let mut out = vec![0.0, 0.0, 0.0];

            // Loop to calculate 12-bit ADC and g value for each axis
            for (i, win) in buf.chunks(2).enumerate() {
                // Combine the two 8 bit registers into one 12-bit number.
                // The registers are left aligned, so right align the 12-bit integer.
                let g = (((win[0] as u16) << 8) | (win[1] as u16)) >> 4;

                // If the number is negative, we have to make it so manually.
                // Transform into negative 2's complement.
                let dim = if win[0] > 0x7F {
                    -(1 + 0xFFF - (g as i16))
                } else {
                    g as i16
                };

                let scale_range = 2.0;
                out[i] = (dim as f64) / ((1 << 11) as f64) * scale_range;
            }

            Ok((out[0], out[1], out[2]))
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

        println!("acceleration: {:?}", acc.read_acceleration());

        let _ = std::io::stdout().flush();
    }
}
