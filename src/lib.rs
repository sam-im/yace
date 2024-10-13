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
        let bitmask: u16 = 0b1111_0000_0000_0000;
        let mut vec = [0; 4];

        for i in 0..4 {
            let masked = (bitmask >> (i * 4)) & opcode;
            // get the next four bits
            let nibble = masked >> ((3 - i) * 4);
            vec[i] = nibble as usize;
        }

        match vec[0] {
            0x0 => match [vec[1], vec[2], vec[3]] {
                [0x0, 0xE, 0x0] => Op::ClearScreen,
                [0x0, 0xE, 0xE] => Op::RetSubroutine,
                _ => panic!("Error decoding opcode: {:4x}", opcode),
            },
            0x1 => Op::Jump((0x0FFF & opcode).into()),
            0x2 => Op::CallSubroutine((0x0FFF & opcode).into()),
            0x3 => Op::SkipIfEqImm(vec[1], (0x00FF & opcode) as u8),
            0x4 => Op::SkipIfNotEqImm(vec[1], (0x00FF & opcode) as u8),
            0x5 => Op::SkipIfEq(vec[1], vec[2]),
            0x6 => Op::SetImm(vec[1], (0x00FF & opcode) as u8),
            0x7 => Op::AddImm(vec[1], (0x00FF & opcode) as u8),
            0x8 => match vec[3] {
                0x0 => Op::Set(vec[1], vec[2]),
                0x1 => Op::Or(vec[1], vec[2]),
                0x2 => Op::And(vec[1], vec[2]),
                0x3 => Op::Xor(vec[1], vec[2]),
                0x4 => Op::Add(vec[1], vec[2]),
                0x5 => Op::Subtract(vec[1], vec[2]),
                0x6 => Op::ShiftRight(vec[1], vec[2]),
                0x7 => Op::SubtractRev(vec[1], vec[2]),
                0xE => Op::ShiftLeft(vec[1], vec[2]),
                _ => panic!("Error decoding opcode: {:4x}", opcode),
            },
            0x9 => Op::SkipIfNotEq(vec[1], vec[2]),
            0xA => Op::SetI((0x0FFF & opcode).into()),
            0xB => Op::JumpWithOffset((0x0FFF & opcode).into()),
            0xC => Op::Random(vec[1], (0x00FF & opcode) as u8),
            0xD => Op::Draw(vec[1], vec[2], vec[3]),
            0xE => match [vec[2], vec[3]] {
                [0x9, 0xE] => Op::SkipIfKeyDown(vec[1]),
                [0xA, 0x1] => Op::SkipIfKeyUp(vec[1]),
                _ => panic!("Error decoding opcode: {:4x}", opcode),
            },
            0xF => match [vec[2], vec[3]] {
                [0x0, 0x7] => Op::GetDelayTimer(vec[1]),
                [0x0, 0xA] => Op::GetKey(vec[1]),
                [0x1, 0x5] => Op::SetDelayTimer(vec[1]),
                [0x1, 0x8] => Op::SetSoundTimer(vec[1]),
                [0x1, 0xE] => Op::AddToIndex(vec[1]),
                [0x2, 0x9] => Op::FontChar(vec[1]),
                [0x3, 0x3] => Op::BCDConv(vec[1]),
                [0x5, 0x5] => Op::StoreMem(vec[1]),
                [0x6, 0x5] => Op::LoadMem(vec[1]),
                _ => panic!("Error decoding opcode: {:4x}", opcode),
            },
            _ => panic!("Error decoding opcode: {:4x}", opcode),
        }
    }

    /// All operation implementations
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
