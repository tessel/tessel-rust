extern crate unix_socket;
use unix_socket::UnixStream;

use std::io::Result as IOResult;
use std::io::Write;
use std::io::Read;
use std::path::Path;

const CMD_ECHO:u8 = 2;
// const CMD_GPIO_IN:u8 = 3;
const CMD_GPIO_HIGH:u8 = 4;
const CMD_GPIO_LOW:u8 = 5;
// const CMD_GPIO_CFG: u8 = 6;
// const CMD_GPIO_WAIT: u8 = 7;
// const CMD_GPIO_INT: u8 = 8;
const CMD_ENABLE_SPI: u8 = 10;
const CMD_DISABLE_SPI: u8 = 11;
const CMD_ENABLE_I2C: u8 = 12;
const CMD_DISABLE_I2C: u8 = 13;
// const CMD_ENABLE_UART: u8 = 14;
// const CMD_DISABLE_UART: u8 = 15;
const CMD_TX: u8 = 16;
const CMD_RX: u8 = 17;
const CMD_TXRX: u8 = 18;
const CMD_START: u8 = 19;
const CMD_STOP: u8 = 20;

pub struct TesselPort {
  sock: UnixStream,
}

impl TesselPort {

  pub fn new(p: &Path) -> TesselPort {
    TesselPort {
      sock: UnixStream::connect(&p).unwrap(),
    }
  }

  pub fn run(&mut self, actions: &mut[Action]) -> IOResult<()> {
    for i in actions.iter() {
      try!(self.sock.write_all(&[i.cmd, i.arg]));
      try!(self.sock.write_all(i.tx));
    }

    for i in actions.iter_mut() {
      // TODO - get an equivalent of read_at_least in here
      try!(self.sock.read(i.rx));
    }

    Ok(())
  }
}

pub struct Action<'a> {
  cmd: u8,
  arg: u8,
  tx: &'a [u8],
  rx: &'a mut [u8]
}

impl<'a> Action<'a>  {

  pub fn echo(tx: &'a [u8], rx: &'a mut [u8]) -> Action<'a> {
    assert!(tx.len() < 256);
    assert_eq!(tx.len(), rx.len());
    Action{ cmd: CMD_ECHO, arg: tx.len() as u8, tx: tx, rx: rx }
  }

  pub fn high(pin: u8) -> Action<'a> {
    Action { cmd: CMD_GPIO_HIGH, arg: pin, tx: &[], rx: &mut [] }
  }

  pub fn low(pin: u8) -> Action<'a> {
    Action { cmd: CMD_GPIO_LOW, arg: pin, tx: &[], rx: &mut [] }
  }

  pub fn enable_spi() -> Action<'a> {
    Action { cmd: CMD_ENABLE_SPI, arg: 0, tx: &[], rx: &mut [] }
  }

  pub fn disable_spi() -> Action<'a> {
    Action { cmd: CMD_DISABLE_SPI, arg: 0, tx: &[], rx: &mut [] }
  }

  pub fn txrx(tx: &'a [u8], rx: &'a mut [u8])  -> Action<'a> {
    assert!(tx.len() < 256);
    assert_eq!(tx.len(), rx.len());
    Action{ cmd: CMD_TXRX, arg: tx.len() as u8, tx: tx, rx: rx }
  }

  pub fn tx(tx: &'a [u8])  -> Action<'a> {
    assert!(tx.len() < 256);
    Action{ cmd: CMD_TX, arg: tx.len() as u8, tx: tx, rx: &mut [] }
  }

  pub fn rx(rx: &'a mut [u8])  -> Action<'a> {
    assert!(rx.len() < 256);
    Action{ cmd: CMD_RX, arg: rx.len() as u8, tx: &[], rx: rx }
  }

  pub fn enable_i2c() -> Action<'a> {
    Action { cmd: CMD_ENABLE_I2C, arg: 0, tx: &[], rx: &mut [] }
  }

  pub fn disable_i2c() -> Action<'a> {
    Action { cmd: CMD_DISABLE_I2C, arg: 0, tx: &[], rx: &mut [] }
  }

  pub fn start(addr: u8) -> Action<'a> {
    Action { cmd: CMD_START, arg: addr, tx: &[], rx: &mut [] }
  }

  pub fn stop() -> Action<'a> {
    Action { cmd: CMD_STOP, arg: 0, tx: &[], rx: &mut [] }
  } 
}