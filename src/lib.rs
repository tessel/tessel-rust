use std::fs::File;
use std::io;
use std::io::Write;

const PORT_A_UDS_PATH: &'static str = "/var/run/tessel/port_a";
const PORT_B_UDS_PATH: &'static str = "/var/run/tessel/port_b";

pub struct Tessel {
    pub port: PortGroup,
    pub led: Vec<LED>,
    pub button: Button
}

impl Tessel {
    pub fn new() -> Tessel {

        let red_led = LED::new("red", "error");
        let amber_led = LED::new("amber", "wlan");
        let green_led = LED::new("green", "user1");
        let blue_led = LED::new("blue", "user2");

        let ports = PortGroup { a: Port::new(PORT_A_UDS_PATH), b: Port::new(PORT_B_UDS_PATH) };

        Tessel {
            port: ports,
            led: vec![red_led, amber_led, green_led, blue_led],
            button: Button {value: false}
        }
    }
}
pub struct PortGroup {
    a: Port,
    b: Port
}
pub struct Port {
    socket_path: &'static str
}

impl Port {
    pub fn new(path: &'static str) -> Port {
        Port {
            socket_path: path
        }
    }
}

pub struct LED {
    color: &'static str,
    kind: &'static str,
    file: File,
    value: bool
}


const LED_PATH_PREFIX: &'static str = "/sys/devices/leds/leds/tessel:";
const LED_PATH_SUFFIX: &'static str = "/brightness";
impl LED {
    pub fn new(color: &'static str, kind: &'static str) -> LED {

        let name = color.to_string() + ":" + kind;
        let path = LED_PATH_PREFIX.to_string() + &name + LED_PATH_SUFFIX;

        let mut led = LED {
            color: color,
            kind: kind,
            value: false,
            file: File::create(path).unwrap()
        };

        led.off().unwrap();

        led
    }

    pub fn on(&mut self)-> Result<(), io::Error> {
        self.write(true)
    }

    pub fn off(&mut self)-> Result<(), io::Error> {
        self.write(false)
    }

    pub fn high(&mut self)-> Result<(), io::Error> {
        self.write(true)
    }

    pub fn low(&mut self)-> Result<(), io::Error> {
        self.write(false)
    }

    pub fn toggle(&mut self)-> Result<(), io::Error> {
        let new_value = !self.value;
        self.write(new_value)
    }

    pub fn read(&self)-> bool {
        self.value
    }

    fn write(&mut self, new_value: bool)-> Result<(), io::Error> {

        self.value = new_value;
        let string_value = match new_value {
            true => b'1',
            false => b'0'
        };

        self.file.write_all(&[string_value])
    }
}

pub struct Button {
    value: bool
}
