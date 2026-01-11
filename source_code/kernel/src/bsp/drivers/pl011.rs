// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! PL011 UART driver.
//!
//! # Resources
//!
//! - <https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf>
//! - <https://developer.arm.com/documentation/ddi0183/latest>

use super::common::MMIODerefWrapper;
use crate::{
    console,
    synchronization::{interface::Mutex, NullLock},
};
use core::{fmt, hint::spin_loop};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// PL011 UART registers.
//
// Descriptions taken from "PrimeCell UART (PL011) Technical Reference Manual" r1p5.
register_bitfields! {
    u32,

    /// Flag Register.
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, LCR_H.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// - If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty.
        /// - This bit does not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is full.
        /// - If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the receive holding register is empty.
        /// - If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) [],

        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
        /// set until the complete byte, including all the stop bits, has been sent from the shift
        /// register.
        ///
        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
        /// the UART is enabled or not.
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// Integer Baud Rate Divisor.
    IBRD [
        /// The integer baud rate divisor.
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud Rate Divisor.
    FBRD [
        ///  The fractional baud rate divisor.
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control Register.
    LCR_H [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers.
        ///
        /// 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register.
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for either UART signals or SIR signals depending on the setting of
        /// the SIREN bit. When the UART is disabled in the middle of reception, it completes the
        /// current character before stopping.
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for either UART signals, or SIR signals depending on the
        /// setting of the SIREN bit. When the UART is disabled in the middle of transmission, it
        /// completes the current character before stopping.
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable:
        ///
        /// 0 = UART is disabled. If the UART is disabled in the middle of transmission or
        /// reception, it completes the current character before stopping.
        ///
        /// 1 = The UART is enabled. Data transmission and reception occurs for either UART signals
        /// or SIR signals depending on the setting of the SIREN bit
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register.
    ICR [
        /// Meta field for all pending interrupts.
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

const DEFAULT_BAUD_RATE: u32 = 921_600;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct Pl011UartInner<const CLOCK_HZ: u32> {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct Pl011Uart<const CLOCK_HZ: u32> {
    inner: NullLock<Pl011UartInner<CLOCK_HZ>>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<const CLOCK_HZ: u32> Pl011UartInner<CLOCK_HZ> {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// Set up baud rate and characteristics.
    ///
    /// This results in 8N1 and 921_600 baud.
    ///
    /// The calculation for the BRD is (we set the clock to 48 MHz in config.txt):
    /// `(48_000_000 / 16) / 921_600 = 3.2552083`.
    ///
    /// This means the integer part is `3` and goes into the `IBRD`.
    /// The fractional part is `0.2552083`.
    ///
    /// `FBRD` calculation according to the PL011 Technical Reference Manual:
    /// `INTEGER((0.2552083 * 64) + 0.5) = 16`.
    ///
    /// Therefore, the generated baud rate divider is: `3 + 16/64 = 3.25`. Which results in a
    /// generated baud rate of `48_000_000 / (16 * 3.25) = 923_077`.
    ///
    /// Error = `((923_077 - 921_600) / 921_600) * 100 = 0.16%`.
    pub fn init(&mut self) {
        // Execution can arrive here while there are still characters queued in the TX FIFO and
        // actively being sent out by the UART hardware. If the UART is turned off in this case,
        // those queued characters would be lost.
        //
        // For example, this can happen during runtime on a call to panic!(), because panic!()
        // initializes its own UART instance and calls init().
        //
        // Hence, flush first to ensure all pending characters are transmitted.
        self.flush();

        // Turn the UART off temporarily.
        self.registers.CR.set(0);

        // Clear all pending interrupts.
        self.registers.ICR.write(ICR::ALL::CLEAR);

        // From the PL011 Technical Reference Manual:
        //
        // The LCR_H, IBRD, and FBRD registers form the single 30-bit wide LCR Register that is
        // updated on a single write strobe generated by a LCR_H write. So, to internally update the
        // contents of IBRD or FBRD, a LCR_H write must always be performed at the end.
        //
        // Set the baud rate, 8N1 and FIFO enabled.
        self.program_baud(DEFAULT_BAUD_RATE);
        self.registers
            .LCR_H
            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

        // Turn the UART on.
        self.registers
            .CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
    }

    fn program_baud(&mut self, baud_rate: u32) {
        let denominator = (16 * baud_rate) as u64;
        let clock = CLOCK_HZ as u64;

        let integer = clock / denominator;
        let remainder = clock % denominator;
        let fractional = ((remainder * 64) + (denominator / 2)) / denominator;

        debug_assert!(integer > 0 && integer < (1 << 16));
        debug_assert!(fractional < 64);

        self.registers
            .IBRD
            .write(IBRD::BAUD_DIVINT.val(integer as u32));
        self.registers
            .FBRD
            .write(FBRD::BAUD_DIVFRAC.val((fractional & 0x3F) as u32));
    }

    /// Send a character.
    #[inline(always)]
    fn write_char(&mut self, c: char) {
        // Spin while TX FIFO full is set, waiting for an empty slot.
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            spin_loop();
        }

        // Write the character to the buffer.
        self.registers.DR.set(c as u32);

        self.chars_written += 1;
    }

    /// Block execution until the last buffered character has been physically put on the TX wire.
    #[inline(always)]
    fn flush(&self) {
        // Spin until the busy bit is cleared.
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            spin_loop();
        }
    }

    /// Retrieve a character.
    #[inline(always)]
    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        // If RX FIFO is empty,
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            // immediately return in non-blocking mode.
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            // Otherwise, wait until a char was received.
            while self.registers.FR.matches_all(FR::RXFE::SET) {
                spin_loop();
            }
        }

        // Read one character.
        let mut ret = self.registers.DR.get() as u8 as char;

        // Convert carrige return to newline.
        if ret == '\r' {
            ret = '\n'
        }

        // Update statistics.
        self.chars_read += 1;

        Some(ret)
    }
}

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
/// used to implement the kernel's UART logging macros. By implementing `write_str()`,
/// we get `write_fmt()` automatically.
///
/// The function takes an `&mut self`, so it must be implemented for the inner struct.
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl<const CLOCK_HZ: u32> fmt::Write for Pl011UartInner<CLOCK_HZ> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl<const CLOCK_HZ: u32> Pl011Uart<CLOCK_HZ> {
    pub const COMPATIBLE: &'static str = "BCM PL011 UART";

    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(Pl011UartInner::new(mmio_start_addr)),
        }
    }

    /// Initialize the hardware block.
    pub fn enable(&self) {
        self.inner.lock(|inner| inner.init());
    }
}

impl<const CLOCK_HZ: u32> console::interface::Write for Pl011Uart<CLOCK_HZ> {
    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
    /// serialize access.
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // Fully qualified syntax for the call to `core::fmt::Write::write_fmt()` to increase
        // readability.
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        // Spin until TX FIFO empty is set.
        self.inner.lock(|inner| inner.flush());
    }
}

impl<const CLOCK_HZ: u32> console::interface::Read for Pl011Uart<CLOCK_HZ> {
    fn read_char(&self) -> char {
        self.inner
            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
    }

    fn clear_rx(&self) {
        self.inner.lock(|inner| {
            while inner
                .read_char_converting(BlockingMode::NonBlocking)
                .is_some()
            {}
        });
    }
}

impl<const CLOCK_HZ: u32> console::interface::Statistics for Pl011Uart<CLOCK_HZ> {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl<const CLOCK_HZ: u32> console::interface::All for Pl011Uart<CLOCK_HZ> {}
