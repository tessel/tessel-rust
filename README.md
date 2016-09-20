# Rust on Tessel

[![Code of Conduct](https://img.shields.io/badge/%E2%9D%A4-code%20of%20conduct-blue.svg?style=flat)](https://github.com/tessel/project/blob/master/CONDUCT.md)

Looking to do some Rust development on Tessel? You’ve found the right place. This repo hosts the Tessel library that provides the hardware API (`gpio.high()`, `spi.transfer()`, etc.) to user applications. It is currently incomplete but being actively worked on. We'd love your help to reach feature parity with the [JavaScript API](https://tessel.io/docs/hardwareAPI)!

**Get in touch**: the team working on Rust for Tessel is all on the #rust-lang channel of the [Tessel Slack ![](https://tessel-slack.herokuapp.com/badge.svg)](https://tessel-slack.herokuapp.com/). Join the conversation!

**Documentation:**

* **[`tessel` crate](https://docs.rs/tessel/)**
* [`accel-mma84` crate](https://docs.rs/accel-mma84/)
* [`ble` crate](https://docs.rs/ble/)
* [`climate-si7020` crate](https://docs.rs/climate-si7020/)
* [`relay-mono` crate](https://docs.rs/relay-mono/)
* [`servo-pca9685` crate](https://docs.rs/servo-pca9685/)

## Quickstart (using remote compilation)

Tessel requires `node` 4.x and `git`. If you haven't yet, install the Tessel 2 CLI using `npm install t2-cli -g`.

To create and deploy the blinking lights example, run the following commands:

```
mkdir rust-fun; cd rust-fun
t2 init --lang=rust
t2 run Cargo.toml
```

The CLI will cross-compile your code on our compilation server, then serve the bundle to your local Tessel. You should see Tessel respond by blinking its user lights alternatingly.

## Tessel Standard Library

The **Tessel Standard Library** is the library that gets 'loaded' into a user program (runs on T2) and presents an API for configuring the hardware (LEDs, module ports, network interfaces, etc.).
You can see the JavaScript version of the Tessel Standard Library [here](https://github.com/tessel/t2-firmware/blob/master/node/tessel-export.js). The most important function is communication with module ports which takes places by writing to a Unix Domain Socket always running on OpenWRT. See [the technical overview](https://github.com/tessel/t2-docs/blob/master/Debugging/Technical_Overview.md) or previously linked JS Standard Library for more detailed information on how that works. Everything sent to the domain socket gets sent to the microcontroller. There is a simple protocol between the MediaTek (running OpenWRT) and the coprocessor to coordinate hardware operations.

Every **Tessel (hardware)** Module has a corresponding software module with the driver code that calls the Standard Library API. The module is managed and installed via a package manager (ie `npm` for Node and `cargo` for Rust). You can see an example of a [JS implementation of our accelerometer module](http://github.com/tessel/accel-mma84). Every hardware module we've made will require its library be ported to Rust and published on crates.io.

## Deploying Rust to Tessel

**tl;dr** Rust can be compiled on a remote cross-compilation server (automatically using `t2 run`), or locally using `rustup` on OSX and Linux. We still lack complete implementations for module ports and Tessel-specific network APIs. See the bottom section for what contributions would be useful.

### Rust Integration Process

In order to use Rust on Tessel, there are several components that are required:

1. Cross Compilation Capabilities - Tessel runs a MIPS architecture which means we have to compile for MIPS instead of our host computer CPU
2. Command Line Interface Deployment - The same CLI used for deploying JS needs a plugin built for deplying to Rust
3. Tessel Standard Library (this repo) - The Tessel-specific library that gives access to the LEDS, button, and module ports
4. Module Libraries - Each Tessel Module needs its library ported from JS to Rust and released as a Cargo Crate

### Remote Compilation Server

See the [rust-compilation-server](https://github.com/tessel/rust-compilation-server/) repo for how to develop for the remote compilation server.

### Local Compilation based on [@badboy's work](https://gist.github.com/badboy/8b951981aaf8bc1700b4703f9c201484) (`rustup`, `git` and `Node` 4.x are requirements)

* `mkdir tessel-rust && cd tessel-rust` to create a folder for Rust projects
* Next we'll setup the linker.

For Linux, first download the OpenWRT SDK with:
```
wget https://s3.amazonaws.com/builds.tessel.io/t2/OpenWRT+SDK/OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2
tar -xf OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2
```
Then add the SDK linker to your path:
```
export PATH=$(readlink -f ./OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64/staging_dir/toolchain-mipsel_24kec+dsp_gcc-4.8-linaro_uClibc-0.9.33.2/bin/):$PATH
```

For OSX, first download the OpenWRT SDK with:
```
wget https://s3.amazonaws.com/builds.tessel.io/t2/OpenWRT+SDK/OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Darwin-x86_64.tar.bz2
tar -xf OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Darwin-x86_64.tar.bz2
```
Then add the linker to your path:
```
export PATH=$(pwd)/openwrt/staging_dir/toolchain-mipsel_24kec+dsp_gcc-4.8-linaro_uClibc-0.9.33.2/bin/:$PATH
```

* Create a test project
```
cargo new --bin hello-rust
cd hello-rust
```
* Create a `.cargo/config` file and add details for cross compilation:
```
[target.mipsel-unknown-linux-gnu]
linker = "mipsel-openwrt-linux-gcc"

[build]
target = "mipsel-unknown-linux-gnu"
```
* Force this project to use the `nightly` version of Rust so we can use an unstable feature to compile properly (temporary)
```
rustup override add nightly
```
* Add the `mipsel` target for compilation:
```
rustup target add mipsel-unknown-linux-gnu
```
* At the top of `src/main.rs`add the following code to ensure we can use the system allocator instead of jmalloc which currently has issues:
```
#![feature(alloc_system)]
extern crate alloc_system;
```

* Build the project (in release mode to optimize the size of the binary)
```
cargo build --release --target=mipsel-unknown-linux-gnu
```
* You can now `scp` it over to your Tessel and run it!
```
scp target/mipsel-unknown-linux-gnu/release/hello-rust root@192.168.1.101:/tmp
ssh -i ~/.tessel/id_rsa root@YOUR_TESSEL_IP ./tmp/hello-rust
```

## Steps to completion and ways you can contribute

Not in any particular order:

- [x] Get another pair of eyes on the cross-compilation server (it's quite simple)
- [x] Help deploy the cross-compilation server to a Digital Ocean droplet
- [ ] Try out the CLI branch that enables Rust deployment with more complicated projects
- [x] Review the CLI branch code
- [ ] Figure out how to use `rustup` with Tessel on Windows
- [ ] Integrate local deployment with `rustup` into the CLI
- [ ] Start writing a Tessel CLI core library in Rust
- [ ] Build out the Tessel Standard Library to allow use of the module ports
- [ ] Figure out how to store the Tessel Standard Library permanently in memory on T2
- [ ] Release the standard library on Crates.io when it's ready
- [ ] Start porting each of the module libraries from JavaScript to Rust (dependent on building out the standard library for Rust)
- [ ] Write [documentation for Rust usage](https://www.github.com/tessel/docs) on Tessel 2

## License

MIT or Apache-2.0, at your option.
