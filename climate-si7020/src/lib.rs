//! https://www.silabs.com/Support%20Documents%2FTechnicalDocs%2FSi7020-A20.pdf

extern crate tessel;

use std::io;
use std::thread;
use std::time::Duration;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Command {
    TempHold = 0xE3,
    RHHold = 0xE5,
    ReadId1 = 0xFA,
    ReadId2 = 0x0F,
    ReadId3 = 0xFC,
    ReadId4 = 0xC9,
}

const TEMPERATURE_OFFSET: f64 = 46.85;
const TEMPERATURE_SLOPE: f64 = 175.72/65536.0;
const HUMIDITY_OFFSET: f64 = 6.0;
const HUMIDITY_SLOPE: f64 = 125.0/65536.0;

const I2C_ID: u8 = 0x40;

#[allow(dead_code)]
pub struct Climate<'a> {
    i2c: tessel::I2cPort<'a>,
    i1: tessel::Pin<'a>,
    i2: tessel::Pin<'a>,
}

impl<'a> Climate<'a> {
    pub fn new<'b>(port: tessel::Port) -> Climate<'b> {
        let (i2c, gpio) = port.i2c();
        let (i1, i2) = gpio.pin_select((5, 6));

        Climate {
            i2c: i2c,
            i1: i1,
            i2: i2,
        }
    }

    /// Reads sequential buffers.
    fn read(&mut self, values: &[Command], buf: &mut [u8]) -> io::Result<()> {
        let a: Vec<u8> = values.iter().map(|x| *x as u8).collect();
        try!(self.i2c.transfer(I2C_ID, &a, buf));
        Ok(())
    }

    fn write(&mut self, values: &[Command], value: u8) -> io::Result<()> {
        let mut a: Vec<u8> = values.iter().map(|x| *x as u8).collect();
        a.push(value);
        self.i2c.send(I2C_ID, &a);
        Ok(())
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let mut buf = [0; 6];
        thread::sleep(Duration::from_millis(30)); //WAKE_UP_TIME
        println!("hi");
        try!(self.read(&[Command::ReadId3, Command::ReadId4], &mut buf));
        println!("hey");
        println!("hi {:?}", buf);
        if buf[0] != 0x14 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid connection code."))
        }
        Ok(())
    }

    pub fn read_temperature(&mut self) -> io::Result<f64> {
        let mut buf = [0; 2];
        try!(self.read(&[Command::TempHold], &mut buf));

        let raw_temp = ((buf[0] as u16) << 8) + (buf[1] as u16);
        let mut temp = ((raw_temp as f64) * TEMPERATURE_SLOPE) - TEMPERATURE_OFFSET;

        // Convert to fahrenheit.
        temp = (temp * (9.0/5.0)) + 32.0;

        Ok(temp)
    }

    pub fn read_humidity(&mut self) -> io::Result<f64> {
        let mut buf = [0; 2];
        try!(self.read(&[Command::RHHold], &mut buf));

        let raw_humidity = ((buf[0] as u16) << 8) + (buf[1] as u16);
        let humidity = ((raw_humidity as f64) * HUMIDITY_SLOPE) - HUMIDITY_OFFSET;

        Ok(humidity)
    }
}
