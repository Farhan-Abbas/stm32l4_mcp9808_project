#!/bin/bash
cd "/home/farhan/Software Engineering/calgary_to_space/stm32l4_mcp9808_project"
cargo build --release && \
probe-rs download --chip STM32L4R5ZITx target/thumbv7em-none-eabihf/release/rustymicrobit && \
probe-rs reset --chip STM32L4R5ZITx
