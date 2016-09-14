//! http://cache.nxp.com/documents/data_sheet/PCA9685.pdf?pspll=1

extern crate tessel;

use std::io;
use std::thread;
use std::time::Duration;
use std::ops::Range;

pub struct RelayArray<'a> {
    pin1: tessel::Pin<'a>,
    pin2: tessel::Pin<'a>,
    states: [bool; 2],
}

impl<'a> RelayArray<'a> {
    pub fn new<'b>(port: tessel::Port) -> RelayArray<'b> {
        //TODO don't use i2c
        let (i2c, gpio) = port.i2c();
        let (pin1, pin2) = gpio.pin_select((5, 6));

        //TODO do we need states or can we read pin output values?
        RelayArray {
            pin1: pin1,
            pin2: pin2,
            states: [false, false],
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        // Set GPIOs as outputs.
        self.pin1.output(false);
        self.pin2.output(false);

        Ok(())
    }

    pub fn set_latch(&mut self, index: usize, value: bool) {
        if index == 1 {
            self.pin1.output(value);
            self.states[0] = value;
        } else if index == 2 {
            self.pin2.output(value);
            self.states[1] = value;
        } else {
            panic!("Invalid relay channel {:?}", index);
        }
    }
}
