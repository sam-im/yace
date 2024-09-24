use log::info;
use std::path::Path;

fn main() {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    info!("Initializing Chip8");
    let mut chip8 = Chip8::new();

    info!("Loading ROM to memory");
    chip8.load_rom(&Path::new("roms/ex.rom"));

    info!("Starting Chip8 emulation");
    // chip8.run();
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

struct Chip8 {
    pc: usize,
    registers: Registers,
    stack: Vec<u16>, // max depth: 16
    memory: Memory,
    timer_delay: u8,
    timer_sound: u8,
    display: [bool; 64 * 32],
}

impl Chip8 {
    fn new() -> Self {
        // initialize ram and load fontset
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

    fn fetch(&mut self) -> u16 {
        // fetch first byte, shift it left
        let opcode: u16 = (self.memory.bytes[self.pc] as u16) << 8;
        // fetch second byte, OR with shifted bytes to place it on the right side
        let opcode: u16 = opcode | (self.memory.bytes[self.pc + 1] as u16);

        self.pc += 2;
        opcode
    }

    fn decode(&self, opcode: u16) -> Operation {
        todo!();
    }

    fn execute(&mut self, op: Operation) {
        todo!();
    }

    fn cycle(&mut self) {
        let opcode = self.fetch();
        let operation = self.decode(opcode);
        self.execute(operation);

        // decrement timers
    }
}

// 16 8-bit registers called V0-VF,
// and an 16-bit instruction register called I
// for storing addresses (12-bit for 0x000-0xFFF)
struct Registers {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8,   // flag register
    i: usize, // 16-bit index register
}

impl Registers {
    fn new() -> Self {
        Self {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,
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

    fn get_byte(&self, offset: &usize) -> u8 {
        self.bytes[*offset]
    }
}

enum Operation {
    ClearScreen,
}
