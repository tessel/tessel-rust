//! http://cache.nxp.com/documents/data_sheet/PCA9685.pdf?pspll=1

extern crate tessel;

use std::io;
use std::ops::Range;
use std::thread;
use std::time::Duration;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Command {
    MODE1 = 0x0,
    LED0_ON_L = 0x06,
    LED0_ON_H = 0x07,
    LED0_OFF_L = 0x08,
    LED0_OFF_H = 0x09,
    PRESCALE = 0xFE,
}

const MAX: u16 = 4096;
//const MODE1 = 0x0;
//const PRE_SCALE = 0xFE;

const I2C_ID: u8 = 0x73;

#[allow(dead_code)]
pub struct ServoArray<'a> {
    i2c: tessel::I2cPort<'a>,
    addr2: tessel::Pin<'a>,
    addr3: tessel::Pin<'a>,
    output_enable: tessel::Pin<'a>,
    range: Range<f64>,
    i2c_id: u8,
}

impl<'a> ServoArray<'a> {
    pub fn new<'b>(port: tessel::Port, addr2: bool, addr3: bool) -> ServoArray<'b> {
        let (i2c, gpio) = port.i2c();
        let (addr2, addr3, output_enable) = gpio.pin_select((5, 6, 7));

        ServoArray {
            i2c: i2c,
            addr2: addr2,
            addr3: addr3,
            output_enable: output_enable,
            range: 0.0..1.0,
            i2c_id: I2C_ID, // TODO
        }
    }

    /// Reads sequential buffers.
    //fn read(&mut self, values: &[Command], buf: &mut [u8]) -> io::Result<()> {
    //    let a: Vec<u8> = values.iter().map(|x| *x as u8).collect();
    //    try!(self.i2c.transfer(I2C_ID, &a, buf));
    //    Ok(())
    //}

    //fn write(&mut self, values: &[Command]) -> io::Result<()> {
    //    let mut a: Vec<u8> = values.iter().map(|x| *x as u8).collect();
    //    a.push(value);
    //    self.i2c.send(I2C_ID, &a);
    //    Ok(())
    //}

    pub fn connect(&mut self) -> io::Result<()> {
        // Enable the outputs.
        self.output_enable.output(false);

        //TODO
        self.addr2.output(false);
        self.addr3.output(false);

        //let mut buf = [0; 6];
        //println!("hi");
        //try!(self.read(&[Command::ReadId3, Command::ReadId4], &mut buf));
        //println!("hey");
        //println!("hi {:?}", buf);
        //if buf[0] != 0x14 {
        //    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid connection code."))
        //}

        self.set_module_frequency(50);

        Ok(())
    }

    pub fn set_module_frequency(&mut self, frequency: u64) {
        let prescale: u8 = (((25000000 / (MAX as u64)) / frequency) - 1) as u8;

        let mut buf = [0; 1];
        self.i2c
            .transfer(self.i2c_id, &[Command::MODE1 as u8], &mut buf);
        let mode = buf[0];

        self.i2c
            .send(self.i2c_id, &[Command::MODE1 as u8, mode | 0x10]);
        self.i2c
            .send(self.i2c_id, &[Command::PRESCALE as u8, prescale]);
        self.i2c.send(self.i2c_id, &[Command::MODE1 as u8, mode]);
        self.i2c.send(self.i2c_id, &[Command::MODE1 as u8, 0xA1]);
    }

    /// Set duty cycle for entry 1 to 16.
    pub fn set_duty_cycle(&mut self, i: usize, value: f64) {
        let offset = ((i - 1) * 4) as u8;
        let reg = (((MAX - 1) as f64) * f64::max(f64::min(value, 1.0), 0.0)) as u16;
        println!(
            "0 0 {:?} {:?}",
            (reg & 0xFF) as u8,
            ((reg >> 8) & 0xFF) as u8
        );
        self.i2c
            .send(self.i2c_id, &[Command::LED0_ON_L as u8 + offset, 0]);
        self.i2c
            .send(self.i2c_id, &[Command::LED0_ON_H as u8 + offset, 0]);
        self.i2c.send(
            self.i2c_id,
            &[Command::LED0_OFF_L as u8 + offset, (reg & 0xFF) as u8],
        );
        self.i2c.send(
            self.i2c_id,
            &[
                Command::LED0_OFF_H as u8 + offset,
                ((reg >> 8) & 0xFF) as u8,
            ],
        );
    }
}
