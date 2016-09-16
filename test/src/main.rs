// Needed for execution on Tessel.
#![feature(alloc_system)]
extern crate alloc_system;

#[macro_use] extern crate nickel;
extern crate accel_mma84;
extern crate local_ip;
extern crate rustc_serialize;
extern crate tessel;

use accel_mma84::Accelerometer;
use nickel::{Nickel, HttpRouter, MediaType, Options};
use rustc_serialize::json;
use std::sync::Mutex;
use tessel::Tessel;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

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
    let sensor = Mutex::new(acc);

    let mut server = Nickel::new();

    // Force four threads.
    server.options = Options::default()
        .thread_count(Some(4));

    server.get("/", middleware! { |_, mut res|
        println!("INDEX");
        res.set(MediaType::Html);
        INDEX_HTML
    });

    server.get("/api/acceleration", middleware! { |_, mut res|
        println!("accel");
        res.headers_mut().set_raw("content-type", vec!["text/event-stream".as_bytes().to_vec()]);

        let mut stream = match res.start() {
            Ok(res) => res,
            _ => panic!("Could not send headers."),
        };

        loop {
            let (x, y, z) = {
                sensor.lock().unwrap().read_acceleration().unwrap()
            };

            let reading = Measurement {
                x: x,
                y: y,
                z: z,
            };

            // Encode using rustc_serialize
            if let Err(_) = stream.write_all(b"data: ") { break; }
            if let Err(_) = stream.write_all(json::encode(&reading).unwrap().as_bytes()) { break; }
            if let Err(_) = stream.write_all(b"\n\n") { break; }
            if let Err(_) = stream.flush() { break; }

            thread::sleep(Duration::from_millis(100));
        }

        println!("done.");
        return Ok(nickel::Halt(stream));
    });

    // Print local IP
    println!("LAN: http://{}/", local_ip::get().unwrap());

    server.listen("0.0.0.0:80");
}
