mod gpio;
mod uart;

use crate::{
    bsp::drivers_interface::BoardDriver,
    console,
};
use crate::console::interface::Write as ConsoleWrite;
use crate::bsp::raspberrypi::dtb::Summary as DtbSummary;
use gpio::init as gpio_init;

pub struct RaspberryPi4bDrivers;

pub static BOARD: RaspberryPi4bDrivers = RaspberryPi4bDrivers;

fn matches_model(summary: &DtbSummary) -> bool {
    summary.model.contains("Raspberry Pi 4")
        || summary
            .compatibles
            .iter()
            .any(|compat| compat.contains("raspberrypi,4"))
}

impl BoardDriver for RaspberryPi4bDrivers {
    fn matches(&self, summary: &DtbSummary) -> bool {
        matches_model(summary)
    }

    unsafe fn init(&'static self, summary: &DtbSummary) -> Result<(), &'static str> {
        let uart_base = summary.peripherals.uart_pl011 as usize;
        if uart_base == 0 {
            return Err("DTB did not provide a PL011 UART base address");
        }

        let gpio_base = summary.peripherals.gpio as usize;
        if gpio_base == 0 {
            return Err("DTB did not provide a GPIO base address");
        }

        gpio_init(gpio_base);
        uart::init(uart_base)?;
        uart::driver().write_char('>');
        uart::driver().write_char('\n');
        Ok(())
    }

    fn board_name(&self) -> &'static str {
        "Raspberry Pi 4 Model B"
    }

    fn default_core_count(&self) -> usize {
        4
    }

    fn uart(&self) -> &'static dyn console::interface::All {
        uart::driver()
    }
}
