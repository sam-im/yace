// TODO keyboard functionality
// TODO optional behaviours for some ops

use chip8::Chip8;
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = args
        .get(1)
        .expect("path to the rom file should be given as first argument");

    let mut chip8 = Chip8::new();
    chip8.load_rom_from_file(Path::new(rom_path));
    chip8.run();
}
