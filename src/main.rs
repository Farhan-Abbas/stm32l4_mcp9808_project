#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4::stm32l4r5;

#[entry]
fn main() -> ! {
    let dp = stm32l4r5::Peripherals::take().unwrap();
    
    // Enable clocks
    dp.RCC.apb1enr1.modify(|_, w| w.pwren().set_bit().i2c1en().set_bit());
    dp.RCC.apb1enr2.modify(|_, w| w.lpuart1en().set_bit());
    dp.RCC.ahb2enr.modify(|_, w| w.gpiogen().set_bit().gpioben().set_bit());
    dp.PWR.cr2.modify(|_, w| w.iosv().set_bit());

    // UART setup (PG7)
    dp.GPIOG.moder.modify(|_, w| w.moder7().alternate());
    dp.GPIOG.afrl.modify(|_, w| w.afrl7().af8());
    dp.LPUART1.brr.write(|w| unsafe { w.bits(106667) });
    dp.LPUART1.cr1.modify(|_, w| w.ue().set_bit().te().set_bit());

    // I2C setup (PB8=SCL, PB9=SDA)
    dp.GPIOB.moder.modify(|_, w| w.moder8().alternate().moder9().alternate());
    dp.GPIOB.otyper.modify(|_, w| w.ot8().open_drain().ot9().open_drain());
    dp.GPIOB.afrh.modify(|_, w| w.afrh8().af4().afrh9().af4());
    dp.I2C1.timingr.write(|w| w.presc().bits(3).sclh().bits(19).scll().bits(25));
    dp.I2C1.cr1.modify(|_, w| w.pe().set_bit());

    // Print function
    let print = |msg: &[u8]| {
        for &b in msg {
            while dp.LPUART1.isr.read().txe().bit_is_clear() {}
            dp.LPUART1.tdr.write(|w| w.tdr().variant(b as u16));
        }
    };

    loop {
        // Clear any I2C flags first
        dp.I2C1.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
        
        // Write register 0x05
        dp.I2C1.cr2.write(|w| w.sadd().bits(0x30).nbytes().bits(1).autoend().set_bit().start().set_bit());
        
        // Wait for TXIS with timeout
        let mut timeout = 10000;
        while dp.I2C1.isr.read().txis().bit_is_clear() && timeout > 0 {
            timeout -= 1;
        }
        
        if timeout > 0 {
            dp.I2C1.txdr.write(|w| w.txdata().variant(0x05));
            
            // Wait for STOP with timeout
            timeout = 10000;
            while dp.I2C1.isr.read().stopf().bit_is_clear() && timeout > 0 {
                timeout -= 1;
            }
            dp.I2C1.icr.write(|w| w.stopcf().set_bit());
            
            // Small delay between transactions
            for _ in 0..1000 { cortex_m::asm::nop(); }
            
            // Read 2 bytes
            dp.I2C1.cr2.write(|w| w.sadd().bits(0x30).rd_wrn().set_bit().nbytes().bits(2).autoend().set_bit().start().set_bit());
            
            // Read first byte with timeout
            timeout = 10000;
            while dp.I2C1.isr.read().rxne().bit_is_clear() && timeout > 0 {
                timeout -= 1;
            }
            
            if timeout > 0 {
                let msb = dp.I2C1.rxdr.read().rxdata().bits();
                
                // Read second byte with timeout
                timeout = 10000;
                while dp.I2C1.isr.read().rxne().bit_is_clear() && timeout > 0 {
                    timeout -= 1;
                }
                
                if timeout > 0 {
                    let lsb = dp.I2C1.rxdr.read().rxdata().bits();
                    
                    // Wait for STOP with timeout
                    timeout = 10000;
                    while dp.I2C1.isr.read().stopf().bit_is_clear() && timeout > 0 {
                        timeout -= 1;
                    }
                    dp.I2C1.icr.write(|w| w.stopcf().set_bit());

                    // Calculate temperature
                    let temp = (((msb as u16) << 8) | (lsb as u16)) & 0x0FFF;
                    let temp_c = (temp as f32) * 0.0625;
                    let int_part = temp_c as u32;
                    let dec_part = ((temp_c - int_part as f32) * 10.0) as u32;
                    
                    // Print temperature
                    print(b"Temperature: ");
                    if int_part >= 10 {
                        print(&[b'0' + (int_part / 10) as u8, b'0' + (int_part % 10) as u8]);
                    } else {
                        print(&[b'0' + int_part as u8]);
                    }
                    print(&[b'.', b'0' + dec_part as u8, b'C', b'\n']);
                } else {
                    print(b"E3\n"); // Second byte timeout
                }
            } else {
                print(b"E1\n"); // Read timeout error
            }
        } else {
            print(b"E2\n"); // Write timeout error
        }
        
        // Wait
        for _ in 0..1000000 { cortex_m::asm::nop(); }
    }
}