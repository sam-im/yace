use log::{error, info};
use std::{path::Path, thread::{self, sleep}, time::Duration};

fn main() {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    info!("Initializing Chip8");
    let mut chip8 = Chip8::new();

    info!("Loading ROM to memory");
    chip8.load_rom(&Path::new("./roms/1-chip8-logo.ch8"));

    info!("Starting Chip8 emulation");
    loop {
        // attempt to simulate 1Mhz
        thread::sleep(Duration::from_millis(1));
        chip8.cycle();
        chip8.print_display();
    }
}

// First 512-bytes are reserved for the interpreter
const START_ADDR: usize = 0x200;
// Each glyphs is represented by 5 bytes
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
// Can be placed anywhere in the first 512 bytes
// But for some reason 0x050 is popular
const FONTSET_START_ADDR: usize = 0x50;

pub struct Chip8 {
    pc: usize,
    registers: Registers,
    stack: Vec<usize>, // max depth: 16
    memory: Memory,
    timer_delay: u8,
    timer_sound: u8,
    pub display: [bool; 64 * 32],
}

impl Chip8 {
    fn new() -> Self {
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
        }
    }

    fn load_rom(&mut self, path: &Path) {
        let rom = std::fs::read(path).unwrap();
        self.memory.load_bytes(&rom, &START_ADDR);
    }

    fn print_display(&self) {
        // clear terminal
        print!("{}[2J", 27 as char);

        for i in 0..32 {
            for j in 0..64 {
                let pixel = if self.display[64 * i + j] { 'o' } else { '.' };
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

    fn decode(&self, opcode: u16) -> Operation {
        let bitmask: u16 = 0b1111_0000_0000_0000;
        let mut vec = [0 as usize; 4];

        for i in 0..=3 {
            let masked = (bitmask >> i * 4) & opcode;
            let nibble = masked >> (3 - i) * 4;
            vec[i] = nibble as usize;
        }

        match vec[0] {
            0x0 => match [vec[1], vec[2], vec[3]] {
                [0x0, 0xE, 0x0] => Operation::ClearScreen,
                [0x0, 0xE, 0xE] => Operation::RetSubroutine,
                _ => panic!("opcode: {}", opcode),
            }
            0x1 => Operation::Jump((0x0FFF & opcode).into()),
            0x2 => Operation::CallSubroutine((0x0FFF & opcode).into()),
            0x6 => Operation::Set(vec[1], (0x00FF & opcode) as u8),
            0x7 => Operation::Add(vec[1], (0x00FF & opcode) as u8),
            0xA => Operation::SetI((0x0FFF & opcode).into()),
            0xD => Operation::Draw(vec[1], vec[2], vec[3]),
            _ => panic!("Couldn't decode opcode: {}", opcode),
        }
    }

    /// All operation implementations
    fn execute(&mut self, op: Operation) {
        match op {
            Operation::ClearScreen => self.display.fill(false),
            Operation::Add(vx, val) => self.registers.v[vx as usize] += val,
            Operation::Draw(vx, vy, row) => {
                let x_coord = (self.registers.v[vx] % 64) as usize;
                let y_coord = (self.registers.v[vy] % 32) as usize;
                self.registers.v[15] = 0;
                let sprite_addr = self.registers.i;
                let bitmask: u8 = 0b1000_0000;

                for i in 0..=row {
                    let sprite_byte = self.memory.get_byte(&sprite_addr + i);
                    for j in 0..8 {
                        if x_coord + j > 64 {
                            break;
                        }
                        let display_index = 64 * (y_coord + i) + x_coord + j;
                        let current_pixel = self.display[display_index];
                        // gets the value of a single bit from the row
                        let sprite_pixel: bool = ((bitmask >> j) & sprite_byte).count_ones() > 0;

                        if current_pixel & sprite_pixel {
                            self.display[display_index] = false;
                            self.registers.v[15] = 1;
                        } else {
                            self.display[display_index] = sprite_pixel;
                        }
                    }
                    if y_coord + i > 32 {
                        break;
                    }
                }
            },
            Operation::Jump(addr) => self.pc = addr,
            Operation::Set(vx, val) => self.registers.v[vx] = val,
            Operation::SetI(addr) => self.registers.i = addr,
            Operation::CallSubroutine(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
            },
            Operation::RetSubroutine => self.pc = self.stack.pop().unwrap(),
        }
    }

    fn cycle(&mut self) {
        let opcode = self.fetch();
        let operation = self.decode(opcode);
        self.execute(operation);

        // check to decrement timers
    }
}

// 16 8-bit registers called V0-VF,
// and an 16-bit instruction register called I
// for storing addresses (12-bit for 0x000-0xFFF)
struct Registers {
    v: [u8; 16],
    i: usize, // 16-bit index register
}

impl Registers {
    fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
        }
    }
}

// Chip8's 4096-bytes of memory
struct Memory {
    bytes: [u8; 4096],
}

impl Memory {
    fn new() -> Self {
        let bytes = [0 as u8; 4096];

        Self { bytes }
    }

    fn load_bytes(&mut self, bytes: &[u8], offset: &usize) {
        for (i, byte) in bytes.iter().enumerate() {
            self.bytes[offset + i] = *byte;
        }
    }

    fn get_byte(&self, offset: usize) -> u8 {
        self.bytes[offset]
    }
}

enum Operation {
    /// 00E0
    ClearScreen,
    /// 1NNN
    Jump(usize),
    /// 6XNN
    Set(usize, u8),
    /// 7XNN
    Add(usize, u8),
    /// ANNN
    SetI(usize),
    /// DXYN
    Draw(usize, usize, usize),
    /// 2NNN: Call a subroutine
    CallSubroutine(usize),
    /// 00EE: Return from subroutine
    RetSubroutine,
}
