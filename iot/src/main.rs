// Needed for execution on Tessel.
#![feature(alloc_system)]
extern crate alloc_system;

#[macro_use] extern crate nickel;
extern crate rustc_serialize;
extern crate accel_mma84;
extern crate tessel;

use accel_mma84::Accelerometer;
use tessel::Tessel;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;
use std::collections::BTreeMap;
use nickel::status::StatusCode;
use nickel::{Nickel, JsonBody, HttpRouter, MediaType};
use rustc_serialize::json::{self, Json, ToJson};
use std::sync::{Arc, Mutex};

#[derive(RustcDecodable, RustcEncodable)]
struct Measurement {
    x: f64,
    y: f64,
    z: f64,
}


fn main() {
    // Create a new Tessel
    let (port_a, _)  = Tessel::ports().unwrap();

    let mut acc = Accelerometer::new(port_a);
    acc.connect().expect("Could not connect to accelerometer.");
    println!("Connected!");

    // Turn on one of the LEDs
    let mut tessel = Tessel::new();
    tessel.led[2].on().unwrap();

    println!("I'm blinking! (Press CTRL + C to stop)");

    let mut server = Nickel::new();

    let sensor = Mutex::new(acc);

    server.get("/", middleware! { |req, mut res|
        let (x, y, z) = sensor.lock().unwrap().read_acceleration().unwrap();

        let reading = Measurement {
            x: x,
            y: y,
            z: z,
        };

        // Encode using rustc_serialize
        res.set(MediaType::Json);
        json::encode(&reading).unwrap()
    });

    server.listen("127.0.0.1:6767");
}
