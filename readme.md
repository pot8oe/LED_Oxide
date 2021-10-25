# LED Oxide
LED Oxide is an HTTP API to devices running the LEDStripController firmware. As
LedStripController devices are only controlled via a USB / serial interface, LED
Oxide's goal is to provide a lightweight network control layer.


# LedStripController
LedStripController is a firmware that controls RGB LED Strip lighting via an
ASCII protocol over serial interface. The LedStripController source can be found
at: https://github.com/pot8oe/ledstripcontroller .


# Hardware Requirements
LED Oxide is only useful if you have a device running LedStripController firmware,
which should run on most Arduino hardware but has primarily been tested on
Teensy 3.2 devices:
https://www.pjrc.com/store/teensy32.html


# Build
The following has been tested on Ubuntu Linux but should be adaptable on other
Linux distributions. It is also assumed Rust language tools have been installed.


## Build - Rust Cargo
Build using the standard rust tools.

1. Clone repository

    `git clone --recurse-submodules git@github-pot8oe.com:pot8oe/led_oxide.git && cd led_oxide`

1. LED Oxide uses Rocket which requires rust nightly.

    `rustup default nightly`

1. Make sure rust is at the latest

    `rustup update`

1. Install libdev development package.

    `sudo apt update && sudo apt upgrade -y sudo apt install -y libudev-dev`

1. Run LED Oxide

    `cargo run`

1. If you want to develop and run led_oxide unit tests.

    `cargo test`
    
1. Some tests require hardware running LedStripController firmware to be
accessible. Make sure an LedStripController based device is plugged in 
and run.

    `cargo test -- --ignored`


## Build - Docker Image
Build a docker image.

1. Clone repository

    `git clone git@github-pot8oe.com:pot8oe/led_oxide.git && cd led_oxide`

1. Change into source directory

    `cd led_oxide`

1. Build docker Image
    `docker build -t led_oxide:0.1 .`


## Run with Docker
Obtain the docker image and use the following command to run container.

    `docker run -p 127.0.0.1:8000:8000/tcp --device=/dev/ttyACM0 -t led_oxide_wip_0.1 & `

* LED Oxide default port is 8000 so we have to export the container port to the
host with:
    `-p 127.0.0.1:8000:8000/tcp`

* Container needs access to the LedStripController device's serial port.
    `--device=/dev/ttyACM0`
