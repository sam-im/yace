// TODO divide main and lib
// TODO keyboard functionality
// TODO left corner of the display tends to skip a few pixels, find why and fix
// TODO configurable loop i.e. a run function
// TODO test and fix some of the operations
// TODO optional behaviours for some ops

use std::{env, path::Path, thread::sleep, time::Duration};
use chip8::Chip8;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_path = args.get(1).expect("path to the rom file should be given as first argument");

    let mut chip8 = Chip8::new();
    chip8.load_rom(Path::new(rom_path));

    loop {
        // attempt to simulate 1Mhz
        sleep(Duration::from_millis(1));
        chip8.cycle();
        chip8.print_display();
    }
}
