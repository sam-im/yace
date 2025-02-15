# yace - Yet Another Chip8 Emulator

# Project Structre
## chip8-emulator
Library that implements the CHIP8 itself. It also includes a main.rs file, which should be ignored as it is only used for simple testing using the terminal. 

### Dependencies
- fastrand: random number generator used for an instruction that needs randomness.

## chip8-wasm
Frontend for the CHIP8 implementation. Uses WASM.

### Dependencies
TODO

## ROMs
The roms folder includes a number of tests from Timendus's [chip8-test-suite repository](https://github.com/Timendus/chip8-test-suite).

# References
- [CHIP-8 Tecnihcal Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference)
