#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;

#[entry]
fn main() -> ! {
    let dp = stm32l4r5::Peripherals::take().unwrap();
    let pwr = dp.PWR;
    let rcc = dp.RCC;
    let gpiog = dp.GPIOG;
    let lpuart1 = dp.LPUART1;

    // Enable power interface and VddIO2
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());
    pwr.cr2.modify(|_, w| w.iosv().set_bit()); // Enable VddIO2

    // Enable GPIOG and LPUART1 clocks
    rcc.ahb2enr.modify(|_, w| w.gpiogen().set_bit());
    rcc.apb1enr2.modify(|_, w| w.lpuart1en().set_bit());

    // Configure PG7 as AF8 (LPUART1_TX)
    gpiog.moder.modify(|_, w| w.moder7().alternate());
    gpiog.otyper.modify(|_, w| w.ot7().clear_bit());
    gpiog.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());
    gpiog.afrl.modify(|_, w| w.afrl7().af8()); // AF8 for LPUART1

    // Calculate BRR for 9600 baud at 4 MHz (MSI default)
    // BRR = (256 * f_clk) / baudrate = (256 * 4_000_000) / 9600 = 106667
    lpuart1.brr.write(|w| unsafe { w.bits(106667) });

    // Enable LPUART1 and TX
    lpuart1.cr1.modify(|_, w| w.ue().set_bit().te().set_bit());

    // Send message
    let msg = b"Hello from LPUART1 on PG7!\n";
    loop {
        for &b in msg {
            while lpuart1.isr.read().txe().bit_is_clear() {}
            lpuart1.tdr.write(|w| unsafe { w.tdr().bits(b as u16) });
        }
    }
}
