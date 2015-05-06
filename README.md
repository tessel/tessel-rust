# Rust on Tessel

Looking to do some Rust development on Tessel? You’ve found the right place.

### The Goal

The end goal of is to make it possible to control our hardware with Rust. That means taking our current API’s (in Javascript) and getting them ported over. Not only does core functionality like bus communication have to work, but all the modules as well! Fun times ahead :)

### Current State

The current state of developing Rust on Tessel is outlined below. There’s plenty of work to do, but together we can make Rust on Tessel happen soon.

Right now we’re working on getting the core Tessel features ported as well as the getting the Accelerometer demo working. Along with this, we’re working on making it super easy to use, so that running can be as simple as `tessel run test.rs`.

### Checklist

This is the outline for what’s done, what needs to be done, and what’s being done. There are many ways to skin a cat (why is that still a phrase?), but I’m keeping it simple. This is the central repo for Rust on Tessel. It'll have links to all the modules devices as well.

##### Core functionality
| Feature | State | Link |
|----------|-------------|------|
| Tessel core functionality | In Progress (as needed) | https://github.com/tessel/rust-tessel |
| Rust CLI Update | Not Started | https://github.com/tessel/t2-cli |
| Cargo on Tessel | In Progress | https://github.com/tessel/rust-tessel |

##### Modules
| Feature | State | Link |
|----------|-------------|------|
| Accelerometer Module | Almost Done | https://github.com/tessel/rust-accel-mma84 |
| Relay Module | Not Started |  |
| Infrared Module | Not Started |  |
| RFID Module | Not Started |  |
| Ambient Module | Not Started |  |
| Servo Module | Not Started |  |
| Climate Module | Not Started |  |

##### Stretch Goals

Calling all Rustaceans. Up for a challenge? These are some stretch goals we would love to get working. They’re not specific to Tessel. These libraries may exist already, and if they don’t, then they would be great contributions not only to Tessel, but to the world of Rust! Want to get involved? Jump in!

| Feature (via USB) | State | Link |
|----------|-------------|------|
| Storage device | Not Started |  |
| Audio device | Not Started |  |
| Camera device | Not Started |  |
| Bluetooth device | Not Started |  |
| Cellular device | Not Started |  |
| GPS device | Not Started |  |

# Getting Started

So you want to develop some part of this yourself? Awesome! This section will help you from setup to actually running your code on a device. So let’s dive in!

Important! - use `Cargo` to create your repository. If you’re not sure what I mean, read more [here](http://doc.crates.io/#let's-get-started).

### Setup

Setup is simply how to get everything configured in order to start developing your library.

Note: this section needs to be updated to use t2-vm, and also have separate setups for MAC and Linux users that forces them to build their own (gaurenteed up to date) versions of the SDK and Rust MIPS Libraries.

1.	Get Rust if you don't already have it
  ```
  curl -s https://static.rust-lang.org/rustup.sh | sudo sh
  ```

1.	Create a new Cargo project if you don't already have one:
  ```
  cargo init rust-library
  ```
  Note: change `rust-library` to your projects name thoughout the rest of the setup

1.	Get [Vagrant](https://www.vagrantup.com/downloads.html) if you don't already have it

1.	After installing Vagrant, run the following to get your VM setup:
  ```
  mkdir ~/Vagrant/rustvm
  cd ~/Vagrant/rustvm
  vagrant init hashicorp/precise64
  ```

1.	This directory now contains your `Vagrantfile`. Add the following line to this file:
  ```
  config.vm.synced_folder "<explicit path>/rust-library", "/home/vagrant/rust-library"
  ```
  Make sure to replace `<explicit path>` with the full path to your cargo project

1.	To bring up and connect to your VM run the following:
  ```
  vagrant up
  vagrant ssh
  ```
  Now your MIPS VM is running. Make sure you're in your VM terminal and complete the following:

1.	Change into the parent of your shared directory:
  ```
  cd /home/vagrant/
  ```

1.	Get the SDK:
  ```
  wget https://kevinmehall.net/tmp/OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2
  tar -xvf OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2
  ```

1.	Get the Rust MIPS Libraries
  ```
  wget https://kevinmehall.net/tmp/rust-mipsel-libs-809a554fca2d0ebc2ba50077016fe282a4064752.tar.bz2
  tar -xvf rust-mipsel-libs-809a554fca2d0ebc2ba50077016fe282a4064752.tar.bz2
  ```
  Note: if you're getting linker errors, build the rust libs yourself to match your current rust version

1.	Set these environment variable so the toolchain knows where things are:
  ```
  export STAGING_DIR=/home/vagrant/OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64/staging_dir
  export PATH=$PATH:$STAGING_DIR/toolchain-mipsel_24kec+dsp_gcc-4.8-linaro_uClibc-0.9.33.2/bin
  ```

1.	Get Rust on the VM
  ```
  sudo apt-get install curl
  curl -s https://static.rust-lang.org/rustup.sh | sudo sh
  ```

1.	On your HOST device, in a new terminal, plug in your Tessel 2 via USB and run the following:
  ```
  git clone https://github.com/tessel/t2-cli
  npm link --local
  t2 list
  ```
  Now you have root access to your Tessel 2 device where you'll be able to run your program

### Routine

After setup is complete this is what your story is going to look like for awhile:

1.	Modify your code on HOST
1.	Build it on the VM
1.	Copy the binary to the Tessel and run it
1.	Repeat step 1 - 3 until it’s working
1.	Make a PR, and bask in the glory

### Build

Building takes place on the VM in order to be compiled using the MIPS architecture. Make sure to use your VM's terminal in your main Cargo repository, and complete the following:
```
rustc -L ../x86_64-unknown-linux-gnu/stage2/lib/rustlib/mipsel-unknown-linux-gnu/lib --target=mipsel-unknown-linux-gnu -Ctarget-cpu=mips32r2 src/lib.rs -Clinker=mipsel-openwrt-linux-gcc -O
```

Note if you're running into compilation problems, make sure that you're actually in your Cargo repo.

### Run

Running takes place on the device itself in order to see real results.

1.	From your HOST machine, copy the binary (in the shared folder) over to your Tessel using:
  ```
  scp lib root@192.168.***.***:~
  ```
  Note: you can get your Tessel's IP by running `t2 status`

1.	From your Tessel 2 run your program using:
  ```
  ./lib /var/run/tessel/port_a
  ```

### Test

Testing is the exact same process as running. Except when you build, specify the `--test` flag: 
```
rustc -L ../x86_64-unknown-linux-gnu/stage2/lib/rustlib/mipsel-unknown-linux-gnu/lib --target=mipsel-unknown-linux-gnu -Ctarget-cpu=mips32r2 src/lib.rs -Clinker=mipsel-openwrt-linux-gcc -O --test
```
This compiles your unit tests and any integrated tests you have in your `tests` directory into a binary much like regular build did.. Simply follow the Run steps above to actually run your tests.

### Finishing

Wow - if you’ve finished a Rust library for the Tessel Platform, rock on - you’re awesome! Make a PR! Someone needs to review the code and get it merged in. Once they test your code, comment, and you update accordingly (if need be), then your code will be merged in and running live on Tessel :)

### Bugs

Naturally there’s going to be bugs. If there’s a problem with any of the repositories then simply file an [issue](https://github.com/tessel/rust-tessel/issues) with steps to reproduce the bug, expected result, and actual result. Using this format will save us all a lot of time and energy. Better yet, fix it yourself and submit a PR!

# Staying Updated

Obviously, the open source nature of this is going to cause some async issues. I highly recommend you stay up to date on this repo - subscribe, see what people are working on, be active! Make sure that you’re not working on something that’s already being done. If you are, collaborate!

### Contact

If you have any questions go to [the forums](https://forums.tessel.io/). We’re trying to keep all discussion open on the forums, but if you have to reach out to someone, here’s my email: ken@technical.io. I’m happy to chat about what you’re working on, take any feedback, suggestions, or comments, or just talk about life/space.


