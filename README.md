# Rust on Tessel

[![Code of Conduct](https://img.shields.io/badge/%E2%9D%A4-code%20of%20conduct-blue.svg?style=flat)](https://github.com/tessel/project/blob/master/CONDUCT.md)

Looking to do some Rust development on Tessel? You’ve found the right place.

**Get in touch**: the team working on Rust for Tessel is all on the #rust-lang channel of the [Tessel Slack ![](https://tessel-slack.herokuapp.com/badge.svg)](https://tessel-slack.herokuapp.com/). Join the conversation!

### tessel-rust
This repo hosts the Tessel library that provides the hardware API (`gpio.high()`, `spi.transfer()`, etc.) to user applications. It is currently incomplete (see gist above for current status) and could use your help to finish it up. The API should end up being very similar to the [JavaScript API](https://tessel.io/docs/hardwareAPI).

### Overview for using Rust on Tessel

**tl;dr** We are working on getting Rust deployment finalized and merged into the command line interface. Rust can be compiled on a remote cross-compilation server or locally using `rustup` on OSX and Linux. We can deploy simple scripts that output to the console but cannot yet control the module ports or network APIs (see the bottom section for what kind of contributions would be useful).

## Quick Start with Remote Compilation (`git`, and `Node` 4.x` are requirements)
* Install the CLI if you haven't already: `npm install t2-cli -g`
* Create a Rust project with a blinky example: `t2 init --lang=rust` (or `cargo new hello --bin`)
* Deploy to your Tessel with the CLI: `t2 run Cargo.toml`

## Quick Start with Local Compilation based on [@badboy's work](https://gist.github.com/badboy/8b951981aaf8bc1700b4703f9c201484) (`rustup`, `git` and `Node` 4.x are requirements)
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

## Developing on the Remote Cross Compilation Server (`Docker`, `git` and `Node` 4.x are requirements)

* Clone the [cross-compilation server repo](https://github.com/tessel/rust-compilation-server) and checkout the `jon-1.0.0` branch.
* Build the Docker image with `docker build -t rustcc .`, then run it with `docker run -p 49160:8080 rustcc`.
* [Clone](https://github.com/tessel/t2-cli) or install the command line interface: `npm install t2-cli -g`
* Make a new directory and inside, create a new Rust project with `t2 init --lang=rust` (it contains a blinky example)
* Deploy the project with `t2 run Cargo.toml --rustcc=$(docker-machine ip):49160`
* To make changes to the server, open two shell. In the first, get access to the shell of the Docker server with `docker attach DOCKER_ID` (you can get your `DOCKER_ID` from `docker ps`. Make changes to `index.js` and then run `docker cp index.js DOCKER_ID:/usr/src/app`. If you make changes to the Dockerfile, run `docker build .`.

# Entire Rust Integration Process
In order to use Rust on Tessel, there are several components that need to be built out to integrate with the existing system:
1. Cross Compilation Capabilities - Tessel runs a MIPS architecture which means we have to compile for MIPS instead of our host computer CPU
2. Command Line Interface Deployment - The same CLI used for deploying JS needs a plugin built for deplying to Rust
3. Tessel Standard Library (this repo) - The Tessel-specific library that gives access to the LEDS, button, and module ports
4. Module Libraries - Each Tessel Module needs its library ported from JS to Rust and released as a Cargo Crate


## Cross Compilation Capabilities
We got started on building a cross-compilation server several months ago. @kevinmehall did a bunch of work to figure out exactly how to build a Rust binary for the MIPS architecture and that work has been automated into [this Docker script](https://github.com/tessel/rust-compilation-server/blob/master/Dockerfile).
The [cross compilation server](https://github.com/tessel/rust-compilation-server) includes that Dockerfile as well as Node server that presents an single API endpoint for cross-compilation. It receives a POST request that sends a tarred project directory, cross compiles that project, then sends the tarred binary back down to the client.  The server is awaiting [v1.0 to land](https://github.com/tessel/rust-compilation-server/pull/6).
To run it, clone this directory and checkout the `jon-1.0.0` branch, build the Docker image (requires Docker to be installed) with `docker build . -t rustCC`, then run it with `docker run -p 49160:8080 rustCC`.

We would eventually like to use [`rustup`](http://blog.rust-lang.org/2016/05/13/rustup.html) for cross compilation within the CLI. It's better suited for the task because it's an easy pathway for local cross-compilation rather than depending on an internet connection and a long-lived remote cross-compilation server. It is better integrated into existing Rust tools like Cargo. We are at a point where we have `rustup` functional with a single workaround and are in the process of integrating into the CLI. We do not have `rustup` functional on Windows.

## Command Line Inteface Deployment
Tessel projects have traditionally been deployed using a command line tool written in JavaScript (with Node.js). In order to bootstrap Rust development on Tessel, we plan to start deploying Rust as well with the same tool (it can be installed with `npm install t2-cli -g`). Eventually, we'd love to build most of the core functionality in Rust and link to that binary with JavaScript (or in whatever language is being deployed) but that's a stretch goal.
In order to integrate with the CLI, we need to write a deployment plugin. This plugin outlines how to detect Rust programs, any pre-deployment steps and any post-deployment steps. A basic version of this integration has been [completed and is awaiting review](https://github.com/tessel/t2-cli/pull/774). This plugin has only been tested with simple programs (ie `cargo new hello --bin`) and can certainly use more testing.
The CLI will detect a Rust project, bundle it into a tarball, send it to the cross compilation server, and then deploy the resulting binary to an available Tessel.

## Tessel Standard Library
The Tessel Standard Library (this repo) is the library that gets 'loaded' into a user program (runs on T2) and presents an API for configuring the hardware (LEDs, module ports, network interfaces, etc.).
You can see the JavaScript version of the Tessel Standard Library [here](https://github.com/tessel/t2-firmware/blob/master/node/tessel-export.js). The most important function is communication with module ports which takes places by writing to a Unix Domain Socket always running on OpenWRT. See [the technical overview](https://github.com/tessel/onboarding/blob/master/T2-TECHNICAL-OVERVIEW.md) or previously linked JS Standard Library for more detailed information on how that works. Everything sent to the domain socket gets sent to the microcontroller. There is a simple protocol between the MediaTek (running OpenWRT) and the coprocessor to coordinate hardware operations.

## Module Libraries
Every Tessel (hardware) Module has a corresponding software module with the driver code that calls the Standard Library API. The module is managed and installed via a package manager (ie `npm` for Node and `cargo` for Rust).
You can see an example of a JS driver [here with the accelerometer module](http://github.com/tessel/accel-mma84). Every hardware module we've made needs to get ported to Rust and deployed to crates.io

## Ways you can help right now (not in any particular order)
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
