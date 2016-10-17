# Rust on Tessel

[![Code of Conduct](https://img.shields.io/badge/%E2%9D%A4-code%20of%20conduct-blue.svg?style=flat)](https://github.com/tessel/project/blob/master/CONDUCT.md)

**This is a work in progress!** See the [open issues](https://github.com/tessel/tessel-rust/issues) on this repo for what we're working on. Feel free to get involved, or join the #rust-lang channel
on our [Tessel Slack community ![](https://tessel-slack.herokuapp.com/badge.svg)](https://tessel-slack.herokuapp.com/).

## Crates

You can read documentation for Tessel crates published on docs.rs:

* **[`tessel` crate](https://docs.rs/tessel/)**
* [`accel-mma84` crate](https://docs.rs/accel-mma84/)
* [`ble` crate](https://docs.rs/ble/)
* [`climate-si7020` crate](https://docs.rs/climate-si7020/)
* [`relay-mono` crate](https://docs.rs/relay-mono/)
* [`servo-pca9685` crate](https://docs.rs/servo-pca9685/)

The `tessel` crate is all you need to start talking to low-level hardware APIs.

## Quickstart

Rust can be compiled on a remote cross-compilation server or locally.

Local compilation with Tessel requires the Rust, Cargo, and the Tessel 2
CLI >= v0.1.0 (installed via npm):

```
npm install -g t2-cli
cargo tessel sdk install
```

You can then create an deploy a blinking lights example using the CLI:

```
mkdir quickstart
cd quickstart
t2 init --lang=rust
t2 run blinky
```

### Using the Remote Compilation Server

You can use the remote compilation server to compile your code quickly and run
it on Tessel. If you don't have Rust and Cargo installed on your machine, or if
`cargo tessel sdk install` says you are using an unsupported version of Rust or
an unsupported platform, this is the quickest way to get started:

```
mkdir quickstart
cd quickstart
t2 init --lang=rust
t2 run blinky --rustcc
```

The `--rustcc` argument invokes the remote cross-compilation server, which bundles
and then locally deploys code to your Tessel.

## Internals

### Tessel Standard Library

The **Tessel Standard Library** is the library that gets 'loaded' into a user program (runs on T2) and presents an API for configuring the hardware (LEDs, module ports, network interfaces, etc.).
You can see the JavaScript version of the Tessel Standard Library [here](https://github.com/tessel/t2-firmware/blob/master/node/tessel-export.js). The most important function is communication with module ports which takes places by writing to a Unix Domain Socket always running on OpenWRT. See [the technical overview](https://github.com/tessel/t2-docs/blob/master/Debugging/Technical_Overview.md) or previously linked JS Standard Library for more detailed information on how that works. Everything sent to the domain socket gets sent to the microcontroller. There is a simple protocol between the MediaTek (running OpenWRT) and the coprocessor to coordinate hardware operations.

### Remote Compilation Server

See the [rust-compilation-server](https://github.com/tessel/rust-compilation-server/) repo for how to develop for the remote compilation server.

## License

MIT or Apache-2.0, at your option.
