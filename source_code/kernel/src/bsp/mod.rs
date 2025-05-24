// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Conditional reexporting of Board Support Packages.

mod device_driver;

#[cfg(feature = "bsp_rpi4")]
pub mod raspberrypi;

#[cfg(feature = "bsp_rpi4")]
pub use raspberrypi::*;
