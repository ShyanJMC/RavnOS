use crate::bsp::drivers::pl011::Pl011Uart;
use core::mem::MaybeUninit;

const CLOCK_HZ: u32 = 48_000_000;

static mut UART: MaybeUninit<Pl011Uart<{ CLOCK_HZ }>> = MaybeUninit::uninit();

fn uart_instance() -> &'static Pl011Uart<{ CLOCK_HZ }> {
    unsafe { UART.assume_init_ref() }
}

pub fn init(base_addr: usize) -> Result<(), &'static str> {
    unsafe {
        UART.write(Pl011Uart::new(base_addr));
    }
    uart_instance().enable();
    Ok(())
}

pub fn driver() -> &'static Pl011Uart<{ CLOCK_HZ }> {
    uart_instance()
}
