//! # STM32L4R5 MCP9808 Temperature Sensor Project
//! 
//! This is a bare-metal Rust application for the STM32L4R5 microcontroller that:
//! - Reads temperature data from an MCP9808 I2C temperature sensor
//! - Transmits temperature readings via UART every 0.5 seconds
//! - Displays temperature in Celsius with 0.1°C precision
//! 
//! ## Hardware Requirements:
//! - STM32L4R5 microcontroller
//! - MCP9808 temperature sensor (Adafruit breakout board)
//! - UART connection on PG7 (for serial output)
//! 
//! ## Pin Connections:
//! - PG7: UART TX (connect to serial adapter RX)
//! - PB8: I2C1 SCL (connect to MCP9808 SCL)
//! - PB9: I2C1 SDA (connect to MCP9808 SDA)
//! - MCP9808 VDD: 3.3V
//! - MCP9808 GND: Ground
//! 
//! ## Communication Settings:
//! - UART: 9600 baud, 8N1
//! - I2C: ~100kHz, MCP9808 address 0x18 (7-bit)
//! 
//! ## Output Format:
//! Temperature: XX.XC (e.g., "Temperature: 24.6C")
//! Error codes: E1, E2, E3 for various I2C timeouts

#![no_std]  // Don't use standard library (not available in embedded)
#![no_main] // Don't use standard main function (we define our own entry point)

// Import required crates for embedded development
use cortex_m_rt::entry;        // Provides #[entry] macro for main function
use panic_halt as _;           // Halt on panic (simple panic handler)
use stm32l4::stm32l4r5;       // STM32L4R5 peripheral access crate

/// Main entry point for the application
/// The #[entry] macro sets this up as the reset handler
/// The -> ! indicates this function never returns (infinite loop)
#[entry]
fn main() -> ! {
    // ========================================================================
    // PERIPHERAL INITIALIZATION
    // ========================================================================
    
    /// Get access to all STM32L4R5 peripherals
    /// take() ensures only one instance exists (singleton pattern)
    /// unwrap() panics if peripherals were already taken (shouldn't happen)
    let dp = stm32l4r5::Peripherals::take().unwrap();
    
    // ========================================================================
    // CLOCK CONFIGURATION
    // ========================================================================
    
    /// Enable peripheral clocks - STM32 peripherals are disabled by default to save power
    /// Each peripheral needs its clock enabled before it can be used
    
    // APB1 Bus 1 peripherals
    dp.RCC.apb1enr1.modify(|_, w| w
        .pwren().set_bit()      // Power interface clock (needed for voltage scaling)
        .i2c1en().set_bit()     // I2C1 peripheral clock (for MCP9808 communication)
    );
    
    // APB1 Bus 2 peripherals  
    dp.RCC.apb1enr2.modify(|_, w| w
        .lpuart1en().set_bit()  // Low Power UART1 clock (for serial output)
    );
    
    // AHB2 Bus peripherals (GPIO ports)
    dp.RCC.ahb2enr.modify(|_, w| w
        .gpiogen().set_bit()    // GPIO Port G clock (for UART pin PG7)
        .gpioben().set_bit()    // GPIO Port B clock (for I2C pins PB8, PB9)
    );
    
    /// Enable VDDIO2 power supply for GPIO Port G
    /// This is required on STM32L4R5 to use GPIO pins on Port G
    dp.PWR.cr2.modify(|_, w| w.iosv().set_bit());

    // ========================================================================
    // UART CONFIGURATION (9600 BAUD ON PG7)
    // ========================================================================
    
    /// Configure PG7 as UART TX pin
    /// GPIO pins can have multiple functions - we need to set them to "alternate function"
    /// and specify which alternate function to use
    
    // Set PG7 to alternate function mode (not regular GPIO input/output)
    dp.GPIOG.moder.modify(|_, w| w.moder7().alternate());
    
    // Set alternate function 8 (AF8) for PG7, which maps to LPUART1_TX
    // Each pin has multiple alternate functions - AF8 connects PG7 to LPUART1
    dp.GPIOG.afrl.modify(|_, w| w.afrl7().af8());
    
    /// Configure UART baud rate
    /// BRR (Baud Rate Register) = (256 × f_clk) / desired_baud_rate
    /// With 4MHz system clock: BRR = (256 × 4,000,000) / 9600 = 106,667
    /// This gives us exactly 9600 baud
    dp.LPUART1.brr.write(|w| unsafe { w.bits(106667) });
    
    /// Enable UART and transmitter
    /// ue = UART Enable, te = Transmitter Enable
    dp.LPUART1.cr1.modify(|_, w| w
        .ue().set_bit()         // Enable the UART peripheral
        .te().set_bit()         // Enable transmitter (we only send, don't receive)
    );

    // ========================================================================
    // I2C CONFIGURATION (FOR MCP9808 SENSOR)
    // ========================================================================
    
    /// Configure I2C pins PB8 (SCL) and PB9 (SDA)
    /// I2C requires special pin configuration since it uses open-drain outputs
    
    // Set PB8 and PB9 to alternate function mode for I2C
    dp.GPIOB.moder.modify(|_, w| w
        .moder8().alternate()   // PB8 = I2C1_SCL (clock line)
        .moder9().alternate()   // PB9 = I2C1_SDA (data line)
    );
    
    /// Set pins to open-drain output type
    /// I2C requires open-drain because multiple devices can pull the line low
    /// but only pull-up resistors can pull it high
    dp.GPIOB.otyper.modify(|_, w| w
        .ot8().open_drain()     // PB8 open-drain
        .ot9().open_drain()     // PB9 open-drain  
    );
    
    /// Set alternate function 4 (AF4) for I2C1 on pins PB8 and PB9
    /// afrh = Alternate Function Register High (for pins 8-15)
    dp.GPIOB.afrh.modify(|_, w| w
        .afrh8().af4()          // PB8 -> I2C1_SCL
        .afrh9().af4()          // PB9 -> I2C1_SDA
    );
    
    /// Configure I2C timing for ~100kHz operation
    /// These values are calculated for 4MHz system clock to get ~100kHz I2C speed
    /// presc = prescaler, sclh = SCL high time, scll = SCL low time
    dp.I2C1.timingr.write(|w| w
        .presc().bits(3)        // Clock prescaler
        .sclh().bits(19)        // SCL high period  
        .scll().bits(25)        // SCL low period
    );
    
    /// Enable I2C peripheral
    dp.I2C1.cr1.modify(|_, w| w.pe().set_bit());

    // ========================================================================
    // UART PRINT FUNCTION
    // ========================================================================
    
    /// Helper function to send data via UART
    /// Takes a byte slice and transmits each byte serially
    /// This is a closure that captures the UART peripheral
    let print = |msg: &[u8]| {
        for &b in msg {
            // Wait for transmit buffer to be empty (TXE = Transmit Empty)
            // This ensures we don't overwrite data that hasn't been sent yet
            while dp.LPUART1.isr.read().txe().bit_is_clear() {}
            
            // Write byte to transmit data register
            // The UART automatically sends this byte over the TX pin
            dp.LPUART1.tdr.write(|w| w.tdr().variant(b as u16));
        }
    };

    // ========================================================================
    // MAIN APPLICATION LOOP
    // ========================================================================
    
    /// Infinite loop that continuously reads temperature and sends via UART
    /// This loop runs every ~0.5 seconds and performs the following:
    /// 1. Write to MCP9808 register 0x05 (temperature register)
    /// 2. Read 2 bytes of temperature data
    /// 3. Convert raw data to Celsius
    /// 4. Format and transmit via UART
    loop {
        // ====================================================================
        // I2C COMMUNICATION WITH MCP9808
        // ====================================================================
        
        /// Clear any pending I2C flags from previous transactions
        /// stopcf = clear STOP flag, nackcf = clear NACK flag
        /// This ensures we start with a clean I2C state
        dp.I2C1.icr.write(|w| w
            .stopcf().set_bit()     // Clear STOP condition flag
            .nackcf().set_bit()     // Clear NACK flag
        );
        
        // ====================================================================
        // STEP 1: WRITE REGISTER ADDRESS TO MCP9808
        // ====================================================================
        
        /// Start I2C write transaction to MCP9808
        /// MCP9808 I2C address is 0x18 (7-bit), but hardware expects 8-bit address
        /// So we use 0x30 = 0x18 << 1 (shift left to make room for R/W bit)
        dp.I2C1.cr2.write(|w| w
            .sadd().bits(0x30)      // Slave address: 0x18 << 1 = 0x30
            .nbytes().bits(1)       // Number of bytes to send: 1 (register address)
            .autoend().set_bit()    // Automatically send STOP after transfer
            .start().set_bit()      // Start the I2C transaction
        );
        
        /// Wait for TXIS (Transmit Interrupt Status) flag with timeout
        /// TXIS indicates the I2C transmit buffer is ready for data
        /// Timeout prevents infinite waiting if sensor doesn't respond
        let mut timeout = 10000;
        while dp.I2C1.isr.read().txis().bit_is_clear() && timeout > 0 {
            timeout -= 1;
        }
        
        /// Check if we successfully got TXIS (no timeout)
        if timeout > 0 {
            /// Send register address 0x05 to MCP9808
            /// Register 0x05 is the temperature register on MCP9808
            dp.I2C1.txdr.write(|w| w.txdata().variant(0x05));
            
            /// Wait for STOP condition to complete the write transaction
            timeout = 10000;
            while dp.I2C1.isr.read().stopf().bit_is_clear() && timeout > 0 {
                timeout -= 1;
            }
            /// Clear the STOP flag
            dp.I2C1.icr.write(|w| w.stopcf().set_bit());
            
            /// Small delay between write and read transactions
            /// Some I2C devices need time between register write and data read
            for _ in 0..1000 { 
                cortex_m::asm::nop();  // No-operation instruction (small delay)
            }
            
            // ================================================================
            // STEP 2: READ TEMPERATURE DATA FROM MCP9808
            // ================================================================
            
            /// Start I2C read transaction to get temperature data
            /// MCP9808 temperature register returns 2 bytes (16-bit value)
            dp.I2C1.cr2.write(|w| w
                .sadd().bits(0x30)          // Same slave address
                .rd_wrn().set_bit()         // Set read mode (R/W bit = 1)
                .nbytes().bits(2)           // Read 2 bytes (temperature is 16-bit)
                .autoend().set_bit()        // Auto-send STOP after reading
                .start().set_bit()          // Start the read transaction
            );
            
            /// Read first byte (MSB - Most Significant Byte)
            timeout = 10000;
            while dp.I2C1.isr.read().rxne().bit_is_clear() && timeout > 0 {
                timeout -= 1;
            }
            
            if timeout > 0 {
                /// Get the high byte of temperature data
                let msb = dp.I2C1.rxdr.read().rxdata().bits();
                
                /// Read second byte (LSB - Least Significant Byte)
                timeout = 10000;
                while dp.I2C1.isr.read().rxne().bit_is_clear() && timeout > 0 {
                    timeout -= 1;
                }
                
                if timeout > 0 {
                    /// Get the low byte of temperature data
                    let lsb = dp.I2C1.rxdr.read().rxdata().bits();
                    
                    /// Wait for STOP condition to complete read transaction
                    timeout = 10000;
                    while dp.I2C1.isr.read().stopf().bit_is_clear() && timeout > 0 {
                        timeout -= 1;
                    }
                    dp.I2C1.icr.write(|w| w.stopcf().set_bit());

                    // ========================================================
                    // STEP 3: CONVERT RAW DATA TO TEMPERATURE
                    // ========================================================
                    
                    /// Combine MSB and LSB into 16-bit value and extract temperature
                    /// MCP9808 format: 16-bit value where lower 12 bits are temperature
                    /// Mask with 0x0FFF to get only the temperature bits (ignore upper 4 bits)
                    let temp = (((msb as u16) << 8) | (lsb as u16)) & 0x0FFF;
                    
                    /// Convert to Celsius using MCP9808 conversion factor
                    /// Each LSB = 0.0625°C (1/16 degree resolution)
                    let temp_c = (temp as f32) * 0.0625;
                    
                    /// Split temperature into integer and decimal parts for display
                    /// We want to show one decimal place (e.g., 24.6°C)
                    let int_part = temp_c as u32;                                    // Integer part (24)
                    let dec_part = ((temp_c - int_part as f32) * 10.0) as u32;     // Decimal part (6)
                    
                    // ========================================================
                    // STEP 4: FORMAT AND TRANSMIT TEMPERATURE
                    // ========================================================
                    
                    /// Send temperature reading via UART in human-readable format
                    print(b"Temperature: ");
                    
                    /// Format integer part - handle both single digit (5) and double digit (25)
                    if int_part >= 10 {
                        // Two digits: send tens digit, then ones digit
                        print(&[
                            b'0' + (int_part / 10) as u8,  // Tens digit
                            b'0' + (int_part % 10) as u8   // Ones digit
                        ]);
                    } else {
                        // Single digit: just send the digit
                        print(&[b'0' + int_part as u8]);
                    }
                    
                    /// Send decimal point, decimal digit, unit, and newline
                    print(&[
                        b'.',                           // Decimal point
                        b'0' + dec_part as u8,         // Decimal digit
                        b'C',                          // Celsius unit
                        b'\n'                          // Newline for next reading
                    ]);
                    
                } else {
                    /// Error: Timeout waiting for second byte
                    print(b"E3\n");
                }
            } else {
                /// Error: Timeout waiting for first byte  
                print(b"E1\n");
            }
        } else {
            /// Error: Timeout waiting for TXIS (sensor not responding)
            print(b"E2\n");
        }
        
        // ====================================================================
        // DELAY BEFORE NEXT READING
        // ====================================================================
        
        /// Wait approximately 0.5 seconds before next temperature reading
        /// This loop executes 1,000,000 no-operation instructions
        /// At 4MHz, this creates roughly 0.5 second delay
        for _ in 0..1000000 { 
            cortex_m::asm::nop();
        }
    }
}

// ============================================================================
// ERROR CODES REFERENCE
// ============================================================================
//
// E1: Timeout waiting for first temperature byte (RXNE timeout)
// E2: Timeout waiting for TXIS flag (sensor not responding to register write)  
// E3: Timeout waiting for second temperature byte (RXNE timeout)
//
// Normal output format: "Temperature: XX.XC" (e.g., "Temperature: 24.6C")
//
// ============================================================================
// MCP9808 REGISTER MAP
// ============================================================================
//
// 0x05: Temperature Register (16-bit, read-only)
//       - Bits 15-13: Alert flags (ignored)
//       - Bits 12-0:  Temperature data
//       - Resolution: 0.0625°C per LSB
//       - Range: -40°C to +125°C
//
// ============================================================================
// I2C TIMING CALCULATION  
// ============================================================================
//
// For 4MHz system clock targeting ~100kHz I2C:
// - PRESC = 3 (prescaler divides by 4)
// - SCLH = 19 (SCL high time)
// - SCLL = 25 (SCL low time)
// 
// Actual I2C frequency ≈ 4MHz / (4 × (19 + 25)) ≈ 90kHz
//
// ============================================================================