// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Minimal GPIO helper to route UART pins.

use super::common::MMIODerefWrapper;
use crate::synchronization::{self, NullLock};
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

// GPIO registers definitions copied from the original implementation.
register_bitfields! {
    u32,

    GPFSEL1 [
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ],

        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ]
    ],

    GPIO_PUP_PDN_CNTRL_REG0 [
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ],

        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

struct GpioInner {
    registers: Registers,
}

impl GpioInner {
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self {
            registers: Registers::new(base_addr),
        }
    }

    fn configure_uart_pins(&mut self) {
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }
}

pub struct Gpio {
    inner: NullLock<GpioInner>,
}

impl Gpio {
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self {
            inner: NullLock::new(GpioInner::new(base_addr)),
        }
    }

    pub fn map_pl011_uart(&self) {
        self.inner.lock(|inner| inner.configure_uart_pins());
    }
}

use synchronization::interface::Mutex;
