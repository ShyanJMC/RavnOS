# RavnOS kernel

This is the RavnOS's kernel

The kernel is not Unix, but have the objective to provide in the future support for Rust stdlib and Linux MUSL. 

## Target / Objective

- [ ] All files documented and visible with "cargo doc".
- [X] ARM64 bits.
- [ ] Strong focus in safe and performance threads.
- [ ] Strong focus in safe and performance drivers.
- [ ] Strong focus in safe and performance file system.

## Software features

- [X] Self hosted trough dependencies vendored
- [X] BASH script for easy and simple build process (the script; build.sh)
- [X] UART Driver
- [X] Detect and start all cores
- [ ] USB 2 Support
- [ ] USB 3 Support
- [ ] USB C Support
- [ ] Keyboard support
- [ ] C standard library MUSL 
- [ ] Support for Rust Core lib
- [ ] Support for Rust std lib

## Hardware features

- [X] ARM64 support
- [X] Raspberry Pi 3/4 GPIO Support
- [ ] HDMI
- [ ] USB 2 Support
- [ ] USB 3 Support
- [ ] USB C Support

## Derived work

The first steps/commits of this program was based in "05_drivers_gpio_uart" of project;

- https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/tree/master/
