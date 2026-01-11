use crate::bsp::drivers::gpio::Gpio;
use core::mem::MaybeUninit;

static mut GPIO: MaybeUninit<Gpio> = MaybeUninit::uninit();

pub fn init(base_addr: usize) {
    unsafe {
        GPIO.write(Gpio::new(base_addr));
        GPIO.assume_init_ref().map_pl011_uart();
    }
}
