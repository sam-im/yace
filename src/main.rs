use std::{path::Path, thread::sleep, time::Duration};

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom(&Path::new("./roms/1-chip8-logo.ch8"));

    loop {
        // attempt to simulate 1Mhz
        sleep(Duration::from_millis(1));
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
        let mut vec = [0 as usize; 4];

        for i in 0..=3 {
            let masked = (bitmask >> i * 4) & opcode;
            // get the next four bits
            let nibble = masked >> (3 - i) * 4;
            vec[i] = nibble as usize;
        }

        match vec[0] {
            0x0 => match [vec[1], vec[2], vec[3]] {
                [0x0, 0xE, 0x0] => Op::ClearScreen,
                [0x0, 0xE, 0xE] => Op::RetSubroutine,
                _ => panic!("Error decoding opcode: {}", opcode),
            }
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
                0x7 => Op::SubtractRev(vec[1], vec[2]),
                _ => panic!("Error decoding opcode: {}", opcode),
            }
            0x9 => Op::SkipIfNotEq(vec[1], vec[2]),
            0xA => Op::SetI((0x0FFF & opcode).into()),
            0xD => Op::Draw(vec[1], vec[2], vec[3]),
            _ => panic!("Error decoding opcode: {}", opcode),
        }
    }

    /// All operation implementations
    fn execute(&mut self, op: Op) {
        match op {
            Op::ClearScreen => self.display.fill(false),
            Op::AddImm(vx, val) => self.registers.v[vx as usize] += val,
            Op::Draw(vx, vy, row) => {
                let x_coord = (self.registers.v[vx] % 64) as usize;
                let y_coord = (self.registers.v[vy] % 32) as usize;
                self.registers.v[15] = 0;
                let sprite_addr = self.registers.i;
                let bitmask: u8 = 0b1000_0000;

                for i in 0..row {
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
            Op::Jump(addr) => self.pc = addr,
            Op::SetImm(vx, val) => self.registers.v[vx] = val,
            Op::SkipIfEqImm(vx, val) => if self.registers.v[vx] == val { self.pc += 2; },
            Op::SkipIfNotEqImm(vx, val) => if self.registers.v[vx] != val { self.pc += 2; },
            Op::SkipIfEq(vx, vy) => if self.registers.v[vx] == self.registers.v[vy] { self.pc += 2 },
            Op::Set(vx, vy) => self.registers.v[vx] = self.registers.v[vy],
            Op::Or(vx, vy) => {
                let save = self.registers.v[vx] | self.registers.v[vy];
                self.registers.v[vx] = save;
            },
            Op::And(vx, vy) => {
                let save = self.registers.v[vx] & self.registers.v[vy];
                self.registers.v[vx] = save;
            },
            Op::Xor(vx, vy) => {
                let save = self.registers.v[vx] ^ self.registers.v[vy];
                self.registers.v[vx] = save;
            }
            Op::Add(vx, vy) => {
                let x = self.registers.v[vx];
                let y = self.registers.v[vy];
                let overflow = (x + y) as u16 > 255;
                let save = x.wrapping_add(y);
                self.registers.v[15] = if overflow { 1 } else { 0 };
                self.registers.v[vx] = save;
            },
            Op::Subtract(vx, vy) => {
                let x = self.registers.v[vx];
                let y = self.registers.v[vy];

                let underflow = x < y;
                let save = x.wrapping_sub(y);

                self.registers.v[15] = if underflow { 0 } else { 1 };
                self.registers.v[vx] = save;
            },
            Op::SubtractRev(vx, vy) => {
                let x = self.registers.v[vx];
                let y = self.registers.v[vy];

                let underflow = y < x;
                let save = y.wrapping_sub(x);

                self.registers.v[15] = if underflow { 0 } else { 1 };
                self.registers.v[vx] = save;
            }
            Op::SkipIfNotEq(vx, vy) => if self.registers.v[vx] != self.registers.v[vy] { self.pc += 2; },
            Op::SetI(addr) => self.registers.i = addr,
            Op::CallSubroutine(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
            },
            Op::RetSubroutine => self.pc = self.stack.pop().unwrap(),
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

enum Op {
    /// 00E0
    ClearScreen,
    /// 00EE: Return from subroutine
    RetSubroutine,
    /// 1NNN
    Jump(usize),
    /// 2NNN: Call a subroutine
    CallSubroutine(usize),
    /// 3XNN: Skip if VX == NN
    SkipIfEqImm(usize, u8),
    /// 4XNN
    SkipIfNotEqImm(usize, u8),
    /// 5XY0
    SkipIfEq(usize, usize),
    /// 6XNN
    SetImm(usize, u8),
    /// 7XNN
    AddImm(usize, u8),
    /// 8XY0 Set vx to vy
    Set(usize, usize),
    /// 8XY1 Binary OR
    Or(usize, usize),
    /// 8XY2 Binary AND
    And(usize, usize),
    /// 8XY3 Logical XOR
    Xor(usize, usize),
    /// 8XY4 Add
    Add(usize, usize),
    /// 8XY5 Subtract vx - vy
    Subtract(usize, usize),
    /// 8XY7 Subtract vy - vx
    SubtractRev(usize, usize),
    // 8XY6 (optional behaviour)
    //ShiftRight, // TODO
    // 8XYE (optional behaviour)
    //ShiftLeft, // TODO
    /// 9XY0
    SkipIfNotEq(usize, usize),
    /// ANNN
    SetI(usize),
    /// BNNN
    /// CXNN
    /// DXYN
    Draw(usize, usize, usize),
    // EX9E
    // EXA1
    // FX07
    // FX15
    // FX18
    // FX1E
    // FX0A
    // FX29
    // FX33
    // FX55
    // FX65
}
