#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;

#[entry]
fn main() -> ! {
    // Access peripherals
    let pac = stm32l4r5::Peripherals::take().unwrap();
    let rcc = pac.RCC;
    let gpioa = pac.GPIOA;
    let usart2 = pac.USART2;

    // 1. Enable clocks
    rcc.ahb2enr.modify(|_, w| w.gpioaen().set_bit());      // GPIOA
    rcc.apb1enr1.modify(|_, w| w.usart2en().set_bit());    // USART2

    // 2. Set PA2 to alternate function mode (AF7 = USART2_TX)
    gpioa.moder.modify(|_, w| w.moder2().alternate());     // Mode: AF
    gpioa.otyper.modify(|_, w| w.ot2().clear_bit());       // Push-pull
    gpioa.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed()); // Optional: High speed
    gpioa.afrl.modify(|_, w| w.afrl2().af7());             // AF7 = USART2

    // 3. Configure USART2: 9600 baud @ 16 MHz => BRR = 1667
    usart2.brr.write(|w| unsafe { w.bits(1667) });

    // 4. Enable USART2 and TX
    usart2.cr1.modify(|_, w| w.ue().set_bit().te().set_bit());

    loop {    // 5. Send message
        let msg = b"Hello, world!\n";
        for &b in msg {
            while usart2.isr.read().txe().bit_is_clear() {} // Wait for TXE (transmit buffer empty)
            usart2.tdr.write(|w| unsafe { w.tdr().bits(b as u16) });
        }
    }
}
