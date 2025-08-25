# STM32L4 MCP9808 Project

This repository contains firmware and related resources for interfacing the MCP9808 high-accuracy I2C temperature sensor with an STM32L4 microcontroller.

## Overview

The project demonstrates how to use the MCP9808 sensor with STM32L4 series MCUs, providing temperature readings over I2C. It includes initialization routines, data acquisition, and basic error handling.

## Features

- Communication with MCP9808 temperature sensor via I2C
- Periodic temperature sampling
- Basic error handling and status reporting
- Example code for STM32L4 microcontrollers

## Getting Started

### Prerequisites

- STM32L4 series development board
- MCP9808 temperature sensor module
- STM32CubeIDE or compatible toolchain
- I2C connection between STM32L4 and MCP9808

### Building and Flashing

1. Clone this repository:
    ```bash
    git clone https://github.com/Farhan-Abbas/stm32l4_mcp9808_project.git
    ```
2. Open the project in STM32CubeIDE.
3. Build the project.
4. Flash the firmware onto your STM32L4 board.

### I2C Connection Example

| STM32L4 Pin | MCP9808 Pin |
|-------------|-------------|
| SDA         | SDA         |
| SCL         | SCL         |
| GND         | GND         |
| VCC (3.3V)  | VDD         |

## Usage

- After flashing, the STM32L4 MCU will start sampling temperature data from the MCP9808 sensor.
- The temperature readings can be output via UART, displayed on an LCD, or processed as per your application needs.

## Folder Structure

- `Src/` - Main source files
- `Inc/` - Header files
- `Drivers/` - Peripheral and middleware drivers
- `README.md` - This file

## References

- [MCP9808 Datasheet](https://www.microchip.com/wwwproducts/en/MCP9808)
- [STM32L4 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00083560.pdf)

## License

This project is licensed under the MIT License.

---

*Created by Farhan Abbas*