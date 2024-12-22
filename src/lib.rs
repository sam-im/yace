mod display;
mod instructions;
mod keypad;
mod memory;
mod registers;
mod sound;

use display::DisplayBuffer;
use instructions::Op;
use keypad::Keypad;
use memory::Memory;
use memory::ROM_START_ADDR;
use registers::Registers;

use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub struct Chip8 {
    registers: Registers,
    memory: Memory,
    pub display: DisplayBuffer,
    pub keypad: Keypad,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            display: [false; 64 * 32],
            keypad: [false; 16],
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            sleep(Duration::from_millis(1));
            self.cycle();
            self.print_display(); // debugging
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.memory.load_bytes(bytes, &ROM_START_ADDR);
    }

    pub fn load_rom_from_file(&mut self, path: &Path) {
        let rom = std::fs::read(path).unwrap();
        self.memory.load_bytes(&rom, &ROM_START_ADDR);
    }

    pub fn print_display(&self) {
        // clear terminal
        print!("{}[2J", 27 as char);

        for i in 0..32 {
            for j in 0..64 {
                let pixel = if self.display[64 * i + j] { '■' } else { ' ' };
                print!("{}", pixel);
            }
            println!()
        }
    }

    fn fetch(&mut self) -> u16 {
        // fetch first byte, shift it left
        let opcode: u16 = (self.memory.bytes[self.registers.pc] as u16) << 8;
        // fetch second byte, OR with shifted bytes to place it on the right side
        let opcode: u16 = opcode | (self.memory.bytes[self.registers.pc + 1] as u16);

        self.registers.pc += 2;
        opcode
    }

    fn decode(&self, opcode: u16) -> Op {
        Op::new(opcode)
    }

    fn execute(&mut self, op: Op) {
        op.execute(self);
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        let operation = self.decode(opcode);
        self.execute(operation);

        // check to decrement timers
    }
}
