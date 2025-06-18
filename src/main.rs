#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;
use aux11::{entry, iprint, iprintln};

#[entry]
fn main() -> ! {
    let (usart1, _mono_timer, _itm) = aux11::init();

    // Send a single character
    usart1
        .tdr
        .write(|w| w.tdr().bits(u16::from(b'X')) );

    loop {}
}


// hello world in UART