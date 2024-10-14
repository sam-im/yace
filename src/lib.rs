mod display;
mod sound;
mod keypad;
mod instructions;
mod memory;
mod registers;

use memory::Memory;
use registers::Registers;
use memory::{ROM_START_ADDR, FONTSET_START_ADDR, FONTSET};
use instructions::Op;

use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub struct Chip8 {
    pc: usize,
    registers: Registers,
    stack: Vec<usize>, // max depth: 16
    memory: Memory,
    timer_delay: u8,
    timer_sound: u8,
    pub display: [bool; 64 * 32],
    keypad: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = Memory::new();
        memory.load_bytes(&FONTSET, &FONTSET_START_ADDR);

        Self {
            pc: 0x200,
            registers: Registers::new(),
            stack: Vec::new(),
            memory,
            timer_delay: 0,
            timer_sound: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            sleep(Duration::from_millis(1));
            self.cycle();
            self.print_display();   // debugging
        }
    }

    pub fn load_rom(&mut self, path: &Path) {
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
        let opcode: u16 = (self.memory.bytes[self.pc] as u16) << 8;
        // fetch second byte, OR with shifted bytes to place it on the right side
        let opcode: u16 = opcode | (self.memory.bytes[self.pc + 1] as u16);

        self.pc += 2;
        opcode
    }

    fn decode(&self, opcode: u16) -> Op {
        Op::new(opcode)
    }

    fn execute(&mut self, op: Op) {
        op.execute(self);
    }

    fn cycle(&mut self) {
        let opcode = self.fetch();
        let operation = self.decode(opcode);
        self.execute(operation);

        // check to decrement timers
    }
}
