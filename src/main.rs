#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;

#[entry]
fn main() -> ! {
    let dp = stm32l4r5::Peripherals::take().unwrap(); // get access to all the device peripherals
    let pwr = dp.PWR;
    let rcc = dp.RCC;
    let gpiog = dp.GPIOG;
    let lpuart1 = dp.LPUART1;
    // peripherals cannot function without their clocks enabled
    // we will use two buses: apb1 for PWR and LPUART1, and ahb2 for GPIO Port G
    // Note: rcc is a register, apb1enr1, ahb2enr, etc. are fields in that register (like sub registers)
    
    // first, we need to set the PWR clock. this will enable the power interface
    rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());  // enable clock for PWR
    // we need to enable VddIO2 because it's required to give power to the GPIOG pin
    pwr.cr2.modify(|_, w| w.iosv().set_bit()); // Enable VddIO2

    // Enable GPIOG and LPUART1 clocks
    // we need to enable the clocks for GPIOG and LPUART1
    // otherwise they will not work
    // GPIOG is on AHB2 bus, LPUART1 is on APB1 bus
    // we will use the RCC registers to enable these clocks
    rcc.ahb2enr.modify(|_, w| w.gpiogen().set_bit()); // enable clock for GPIOG
    rcc.apb1enr2.modify(|_, w| w.lpuart1en().set_bit()); // enable clock for LPUART1

    // Configure PG7 as AF8 (LPUART1_TX)
    gpiog.moder.modify(|_, w| w.moder7().alternate());  // Set pin 7 to alternate function mode
    gpiog.otyper.modify(|_, w| w.ot7().clear_bit()); // Set pin 7 output type
    gpiog.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed()); // Set pin 7 speed
    gpiog.afrl.modify(|_, w| w.afrl7().af8()); // AF8 for LPUART1
    // moder says PG7 will be controlled by an alternate function.
    // afrl says that alternate function will be AF8 (LPUART1_TX)


    // Calculate BRR for 9600 baud at 4 MHz (MSI default)
    // BRR = (256 * f_clk) / baudrate = (256 * 4_000_000) / 9600 = 106667
    lpuart1.brr.write(|w| unsafe { w.bits(106667) });  // set the baud rate of LPUART1 to 9600 baud

    // Enable LPUART1 and TX
    lpuart1.cr1.modify(|_, w| w.ue().set_bit().te().set_bit());


    // **************************************************************
    // **************************************************************
    // begin mcp9808 stuff
    // **************************************************************
    // **************************************************************




    // **************************************************************
    // **************************************************************
    // end mcp9808 stuff
    // **************************************************************
    // **************************************************************
    // Send message
    let msg = b"Hello from LPUART1 on PG7!\n";
    loop {
        for &b in msg {
            while lpuart1.isr.read().txe().bit_is_clear() {}
            lpuart1.tdr.write(|w| unsafe { w.tdr().bits(b as u16) });
        }
    }
}