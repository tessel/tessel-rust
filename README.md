# Rust on Tessel

Looking to do some Rust development on Tessel? You’ve found the right place.

### Current State

The current state of developing Rust on Tessel is outlined below. There’s plenty of work to do, but together we can make Rust on Tessel happen soon.

### The Goal

The end goal of is to make it possible to control our hardware with Rust. That means taking our current API’s (in Javascript) and getting them ported over. Not only does core functionality like bus communication have to work, but all the modules as well! Fun times ahead :)

### Now

Right now we’re working on getting the core Tessel features ported as well as the getting the Accelerometer demo working. Along with this, we’re working on making it super easy to use, so that running can be as simple as `tessel run test.rs`.

### Checklist

This is the outline for what’s done, what needs to be done, and what’s being done. There are many ways to skin a cat (why is that still a phrase?), but I’m keeping it simple, and having this be the central hub for Rust development on the Tessel platform.

##### Core functionality
| Feature | State | Link |
|----------|-------------|------|
| Tessel core functionality | In Progress (as needed) | https://github.com/johnnyman727/accel-mma84-rust/blob/jon-get-accel/src/tessel.rs |
| Relay Module | Not Started |  |
| Rust CLI Update | Not Started |  |
| Cargo on Tessel | In Progress |  |

##### Modules
| Feature | State | Link |
|----------|-------------|------|
| Accelerometer Module | Almost Done | https://github.com/johnnyman727/accel-mma84-rust |
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

Note - it’s assumed you’re using `Cargo` in your repository. If you’re not sure what I mean, read more [hererere](http://doc.crates.io/#let's-get-started).

### Setup

Setup is simply how to get everything configured in order to start developing your library. Keep in mind there are three parts here - a HOST (your machine), a VM (A machine to compile using MIPS), and a DEVICE (A MIPS device). Here’s a list of steps to run through to make get everything set up:

1.	On your HOST get [Vagrant](https://www.vagrantup.com/downloads.html)
2.	Create and change into a new directory `mkdir ~/Vagrant/rustvm`
3.	Create your MIPS VM by running `vagrant init hashicorp/precise64`
4.	In your `Vagrantfile` add this line `config.vm.synced_folder "<host repo dir>", "/home/vagrant/<your repo name>"` to sync your HOST repo with the VM repo.
5.	Boot up the VM with `vagrant up`
6.	SSH into the VM using `vagrant ssh`
7.	On your VM, change directory into your repository
8.	Mac: \
8.	Linux:
9.	Change back to your HOST terminal, and plug in your Tessel 2 & module
10.	Clone the T2 CLI repo using `git clone https://github.com/tessel/t2-cli`
11.	Change into that dir and run `npm link --local`
12.	In order to see your Tessel 2 run `t2 list` and note the IP address
13.	SSH into your T2 using `ssh root@192.168.***.***`

### Routine

After setup is complete this is what your story is going to look like for awhile:

1.	Modify your code
2.	Compile it on the VM
3.	Copy the binary to the device and run it
4.	Repeat step 1 - 3 until it’s working
5.	Make a PR, and bask in the glory

### Build

Building takes place on the VM in order to be compiled using the MIPS architecture. Make sure to use your VM's terminal, and complete the following:

1.	Change directory into your repository
2.	To build the code run `rustc -L ../x86_64-unknown-linux-gnu/stage2/lib/rustlib/mipsel-unknown-linux-gnu/lib --target=mipsel-unknown-linux-gnu -Ctarget-cpu=mips32r2 src/lib.rs -Clinker=mipsel-openwrt-linux-gcc -O`

### Run

Running takes place on the device itself in order to see real results.

1.	From your HOST machine copy over the compiled binary (in the shared folder) over to your DEVICE using `scp lib root@192.168.***.***:~`
2.	From your Tessel 2 run your program using `./lib /var/run/tessel/port_a` in your home directory

### Test

Testing is like building and running combined. Compiling to test your code will create a binary that runs your unit and integrated tests. To test, run the following:

1.	On your VM, change into your repo directory
2.	To build the code run `rustc -L ../x86_64-unknown-linux-gnu/stage2/lib/rustlib/mipsel-unknown-linux-gnu/lib --target=mipsel-unknown-linux-gnu -Ctarget-cpu=mips32r2 src/lib.rs -Clinker=mipsel-openwrt-linux-gcc -O --test` (note the `--test` flag)
3.	From your HOST, copy over the compiled binary (in the shared folder) over to your DEVICE using `scp lib root@192.168.***.***:~`
4.	From your DEVICE test your program by running `./lib` in your home directory 

### Finishing

Wow - if you’ve finished a Rust library for the Tessel Platform, rock on - you’re awesome! Make a PR! Someone needs to review the code and get it merged in. Once they test your code, comment, and you update accordingly (if need be), then your code will be merged in and running live on Tessel :)

### Bugs

Naturally there’s going to be bugs. If there’s a problem with any of the repositories then simply file an issue with steps to reproduce the bug, expected result, and actual result. Using this format will save us all a lot of time and energy. Even better, fix it yourself and submit a PR!

# Staying Updated

Obviously, the open source nature of this is going to cause some async issues. I highly recommend you stay up to date on this repo - subscribe, see what people are working on, be active! Make sure that you’re not working on something that’s already being done!

### Everybody loves email

I’m considering throwing together a mailing list that I can blast when need be. I wouldn’t spam it, but it would exist to complement the ever changing state of the repo. Occasional updates that would let the subscribers know the current progress, etc.

Curious as to what ya’ll think about this. It would probably be a google group. Suggestions welcome. 

### Contact

If you have any questions go to [the forums](https://forums.tessel.io/). We’re trying to keep all discussion open on the forums, but if you have to reach out to someone, here’s my email: ken@technical.io. I’m happy to chat about what you’re working on, take any feedback, suggestions, or comments, or just talk about life/space.


