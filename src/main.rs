#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;

#[entry]
fn main() -> ! {
    let pac = stm32l4r5::Peripherals::take().unwrap();

    let rcc = pac.RCC;
    // Enable GPIOB instead of GPIOA
    rcc.ahb2enr.modify(|_, w| w.gpioben().set_bit());

    let gpiob = pac.GPIOB;
    // Set PB14 as output
    gpiob.moder.modify(|_, w| w.moder14().output());
    gpiob.otyper.modify(|_, w| w.ot14().clear_bit()); // Push-pull

    loop {
        gpiob.odr.modify(|_, w| w.odr14().set_bit()); // Turn LED on
    }
}
