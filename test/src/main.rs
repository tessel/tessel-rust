// Needed for execution on Tessel.
#![feature(alloc_system)]
extern crate alloc_system;

#[macro_use] extern crate nickel;
extern crate accel_mma84;
extern crate local_ip;
extern crate rustc_serialize;
extern crate tessel;

use accel_mma84::Accelerometer;
use nickel::{Nickel, HttpRouter, MediaType};
use rustc_serialize::json;
use std::sync::Mutex;
use tessel::Tessel;

#[derive(RustcDecodable, RustcEncodable)]
struct Measurement {
    x: f64,
    y: f64,
    z: f64,
}

const INDEX_HTML: &'static str = include_str!("index.html");

fn main() {
    // Create a new Tessel
    let (port_a, _)  = Tessel::ports().unwrap();

    let mut acc = Accelerometer::new(port_a);
    acc.connect().expect("Could not connect to accelerometer.");
    println!("Connected to accelerometer.");

    let mut server = Nickel::new();
    let sensor = Mutex::new(acc);

    server.get("/", middleware! { |_, mut res|
        res.set(MediaType::Html);
        INDEX_HTML
    });

    server.get("/api/acceleration", middleware! { |_, mut res|
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

    // Print local IP
    println!("LAN: http://{}/", local_ip::get().unwrap());

    server.listen("0.0.0.0:80");
}
