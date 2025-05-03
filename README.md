# yace - Yet Another Chip8 Emulator

# Project Structure

## chip8-emulator
Library that implements the CHIP8 itself. 
It also includes a main file for quick testing inside the terminal.

### Dependencies
- `fastrand`

## chip8-wasm
Frontend for the CHIP8 implementation. Uses WebAssembly.

### Dependencies
- `wasm-bindgen`

## ROMs
The roms folder includes a number of tests from Timendus's [chip8-test-suite repository](https://github.com/Timendus/chip8-test-suite).
Hotloading a rom is not implemented. Instead a single test rom is embedded inside the binary.

# Build
Use [`wasm-pack build --target web`](https://github.com/rustwasm/wasm-pack) to build the `chip8-wasm` crate, which will also compile `chip8-emulator` as a dependency. 
Simplest way to run the wasm file locally is to copy the `index.html` inside to the generated pkg directory and run a simple http dev server there, e.g. `python -m http.server`.

# References
- [CHIP-8 Tecnihcal Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference)
