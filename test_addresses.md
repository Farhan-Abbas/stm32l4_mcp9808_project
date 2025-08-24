# MCP9808 Address Analysis

## Standard MCP9808 Addresses
The MCP9808 has a 7-bit I2C address that can be configured with pins A2, A1, A0:

- Base address: 0011000b = 0x18
- A2, A1, A0 can be tied to VDD, GND, or SDA to change address
- Possible addresses: 0x18 to 0x1F

## Addresses found in scan: 0x09

Address 0x09 = 0000 1001b

This is NOT in the MCP9808 range. Possible explanations:

1. **Different sensor**: Could be another I2C temperature sensor
2. **Wiring error**: Wrong device connected
3. **Address shift**: Some I2C implementations use 8-bit addresses (with R/W bit)
4. **Other I2C device**: Could be anything else on the board

## Common I2C Temperature Sensors:
- DS3231 RTC with temp: 0x68
- LM75: 0x48-0x4F  
- TMP102: 0x48-0x4B
- Si7021: 0x40
- SHT30: 0x44 or 0x45

## Next Steps:
1. Try reading device ID from 0x09
2. Test all MCP9808 addresses (0x18-0x1F)
3. Check if 0x09 responds to MCP9808 commands
