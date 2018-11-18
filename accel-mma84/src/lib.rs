//! https://cdn.sparkfun.com/datasheets/Sensors/Accelerometers/MMA8452Q-rev8.1.pdf

extern crate tessel;

use std::io;

#[repr(u8)]
pub enum ScaleRange {
    Scale2G = 0b00,
    Scale4G = 0b01,
    Scale8G = 0b10,
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

#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum SampleRate {
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

#[allow(dead_code)]
pub struct Accelerometer<'a> {
    i2c: tessel::I2cPort<'a>,
    i1: tessel::Pin<'a>,
    i2: tessel::Pin<'a>,
}

impl<'a> Accelerometer<'a> {
    pub fn new<'b>(port: tessel::Port) -> Accelerometer<'b> {
        let (i2c, gpio) = port.i2c();
        let (i1, i2) = gpio.pin_select((5, 6));

        Accelerometer {
            i2c: i2c,
            i1: i1,
            i2: i2,
        }
    }

    fn read_register(&mut self, cmd: Command) -> io::Result<u8> {
        let mut xr: [u8; 1] = [0; 1];
        self.read_registers(cmd, &mut xr)?;
        Ok(xr[0])
    }

    /// Reads sequential buffers.
    fn read_registers(&mut self, cmd: Command, buf: &mut [u8]) -> io::Result<()> {
        self.i2c.transfer(I2C_ID, &[cmd as u8], buf)?;
        Ok(())
    }

    fn write_register(&mut self, cmd: Command, value: u8) -> io::Result<()> {
        self.i2c.send(I2C_ID, &[cmd as u8, value]);
        Ok(())
    }

    pub fn connect(&mut self) -> io::Result<()> {
        if self.read_register(Command::WhoAmI)? != 0x2A {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid connection code.",
            ));
        }

        self.set_scale_range(ScaleRange::Scale2G)?;
        self.set_sample_rate(SampleRate::Rate100)?;

        Ok(())
    }

    fn standby_enable(&mut self) -> io::Result<()> {
        // Sets the MMA8452 to standby mode.
        let value = self.read_register(Command::CtrlReg1)?;
        self.write_register(Command::CtrlReg1, value & !(0x01u8))
    }

    fn standby_disable(&mut self) -> io::Result<()> {
        // Sets the MMA8452 to active mode.
        let value = self.read_register(Command::CtrlReg1)?;
        self.write_register(Command::CtrlReg1, value | (0x01u8))
    }

    pub fn set_scale_range(&mut self, range: ScaleRange) -> io::Result<()> {
        self.standby_enable()?;
        self.write_register(Command::XyzDataCfg, range as u8)?;
        self.standby_disable()?;

        Ok(())
    }

    pub fn set_sample_rate(&mut self, rate: SampleRate) -> io::Result<()> {
        self.standby_enable()?;

        // Clear the three bits of output rate control (0b11000111 = 199)
        let mut value = self.read_register(Command::CtrlReg1)?;
        value &= 0b11000111;
        self.write_register(Command::CtrlReg1, value | ((rate as u8) << 3))?;

        self.standby_disable()?;

        Ok(())
    }

    pub fn read_acceleration(&mut self) -> io::Result<(f64, f64, f64)> {
        let mut buf = [0; 6];
        self.read_registers(Command::OutXMsb, &mut buf)?;

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
