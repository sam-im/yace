use crate::memory::FONTSET_START_ADDR;
use crate::Chip8;

pub enum Op {
    /// 00E0: Clear the screen
    ClearScreen,
    /// 00EE: Return from subroutine
    RetSubroutine,
    /// 1NNN: Jump to address NNN
    Jump(usize),
    /// 2NNN: Call subroutine
    CallSubroutine(usize),
    /// 3XNN: Skip next instruction if register VX == NN
    SkipIfEqImm(usize, u8),
    /// 4XNN: Skip next instruction if register VX =! NN
    SkipIfNotEqImm(usize, u8),
    /// 5XY0: Skip next instruction if register values VX == VY
    SkipIfEq(usize, usize),
    /// 6XNN: Store value NN in register VX
    SetImm(usize, u8),
    /// 7XNN: Add the value NN to register VX
    AddImm(usize, u8),
    /// 8XY0: Set register VX to VY
    Set(usize, usize),
    /// 8XY1: Set register VX to VX OR VY
    Or(usize, usize),
    /// 8XY2: Set register VX to VX AND VY
    And(usize, usize),
    /// 8XY3: Set register VX to VX XOR VY
    Xor(usize, usize),
    /// 8XY4: Add the value of register VY to register VX
    /// Sets register VF to 1 if a carry occurs, if not set it to 0
    Add(usize, usize),
    /// 8XY5: Subtract the value of register VY from register VX
    /// Set register VF to 0 if a borrow occurs, if not set it to 1
    Subtract(usize, usize),
    /// 8XY6: Store the value of register VY shifted right one bit to register VX
    /// Set register VF to the least significant bit prior to the shift
    ShiftRight(usize, usize),
    /// 8XY7: Set register VX to the value of VY - VX
    /// Set VF to 0 if a borrow occurs, if not set it to 01
    SubtractRev(usize, usize),
    /// 8XYE: Store the value of VY shifted left one bit in register VX
    /// Set VF to the most significant bit prior to the shift
    ShiftLeft(usize, usize),
    /// 9XY0: Skip next instruction if the values of register VX =! VY
    SkipIfNotEq(usize, usize),
    /// ANNN: Store memory address NNN in register I
    SetI(usize),
    /// BNNN: Jump to address NNN + V0
    JumpWithOffset(usize),
    /// CXNN: Set register VX to a random number with a mask of NN
    Random(usize, u8),
    /// DXYN: Draw a sprite at position VX, VY with N bytes starting from
    /// the address stored in register I
    /// Set VF to 1 if any previously set pixel is unset, 0 otherwise
    /// Explanation: Trying to set an already set pixel results in it being unset.
    Draw(usize, usize, usize),
    /// EX9E: Skip next instruction if the key with the value in VX is pressed
    SkipIfKeyDown(usize),
    /// EXA1: Skip next instruction if the key with the value in VX is not pressed
    SkipIfKeyUp(usize),
    /// FX07: Set VX to the current value of the delay timer
    GetDelayTimer(usize),
    /// FX0A: Stop execution until a key is pressed, store the key in VX
    GetKey(usize),
    /// FX15: Set the delay timer to the value of VX
    SetDelayTimer(usize),
    /// FX18: Set the sound timer to the value of VX
    SetSoundTimer(usize),
    /// FX1E: Add the value in VX to index register
    AddToIndex(usize),
    /// FX29: Set register I to an address of a font character sprite specified in VX.
    FontChar(usize),
    /// FX33: Store the binary-coded decimal equivalent of the value stored in register
    /// VX to addresses I, I + 1, I + 2.
    BCDConv(usize),
    /// FX55: Store registers V0 to VX inclusive in memory starting at address I,
    /// I is set to I + X + 1 after operation.
    StoreMem(usize),
    /// FX65: Store registers V0 to VX inclusive with the values starting from the address
    /// stored in register I, I is set to I + X + 1 after operation.
    LoadMem(usize),
}

impl Op {
    pub fn new(opcode: u16) -> Self {
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
    pub fn execute(self, chip8: &mut Chip8) {
        match self {
            Op::ClearScreen => chip8.display.fill(false),
            Op::AddImm(vx, val) => chip8.registers.v[vx] = chip8.registers.v[vx].wrapping_add(val),
            Op::Jump(addr) => chip8.registers.pc = addr,
            Op::SetImm(vx, val) => chip8.registers.v[vx] = val,
            Op::SkipIfEqImm(vx, val) => {
                if chip8.registers.v[vx] == val {
                    chip8.registers.pc += 2;
                }
            }
            Op::SkipIfNotEqImm(vx, val) => {
                if chip8.registers.v[vx] != val {
                    chip8.registers.pc += 2;
                }
            }
            Op::SkipIfEq(vx, vy) => {
                if chip8.registers.v[vx] == chip8.registers.v[vy] {
                    chip8.registers.pc += 2
                }
            }
            Op::Set(vx, vy) => chip8.registers.v[vx] = chip8.registers.v[vy],
            Op::Or(vx, vy) => {
                let save = chip8.registers.v[vx] | chip8.registers.v[vy];
                chip8.registers.v[vx] = save;
            }
            Op::And(vx, vy) => {
                let save = chip8.registers.v[vx] & chip8.registers.v[vy];
                chip8.registers.v[vx] = save;
            }
            Op::Xor(vx, vy) => {
                let save = chip8.registers.v[vx] ^ chip8.registers.v[vy];
                chip8.registers.v[vx] = save;
            }
            Op::Add(vx, vy) => {
                let x = chip8.registers.v[vx];
                let y = chip8.registers.v[vy];

                chip8.registers.v[0xF] = 0;

                if x.checked_add(y).is_none() {
                    chip8.registers.v[0xF] = 1;
                };

                let save = x.wrapping_add(y);
                chip8.registers.v[vx] = save;
            }
            Op::Subtract(vx, vy) => {
                let x = chip8.registers.v[vx];
                let y = chip8.registers.v[vy];

                chip8.registers.v[0xF] = 0;

                if x.checked_sub(y).is_none() {
                    chip8.registers.v[0xF] = 1;
                };

                let save = x.wrapping_sub(y);
                chip8.registers.v[vx] = save;
            }
            Op::ShiftRight(vx, vy) => {
                let y = chip8.registers.v[vy];
                chip8.registers.v[0xF] = 0;
                if 0b0000_0001 & y > 0b0 {
                    // check rightmost bit for 1
                    chip8.registers.v[0xF] = 1;
                }
                chip8.registers.v[vx] = y.wrapping_shr(1);
            }
            Op::SubtractRev(vx, vy) => {
                let x = chip8.registers.v[vx];
                let y = chip8.registers.v[vy];

                chip8.registers.v[0xF] = 0;
                if y.checked_sub(x).is_none() {
                    chip8.registers.v[0xF] = 1;
                }
                let save = y.wrapping_sub(x);
                chip8.registers.v[vx] = save;
            }
            Op::ShiftLeft(vx, vy) => {
                let y = chip8.registers.v[vy];
                chip8.registers.v[0xF] = 0;
                if 0b1000_0000 & y > 0b0 {
                    // check leftmost bit for 1
                    chip8.registers.v[0xF] = 1;
                }
                chip8.registers.v[vx] = y << 1;
            }
            Op::SkipIfNotEq(vx, vy) => {
                if chip8.registers.v[vx] != chip8.registers.v[vy] {
                    chip8.registers.pc += 2;
                }
            }
            Op::SetI(addr) => chip8.registers.i = addr,
            Op::JumpWithOffset(addr) => chip8.registers.pc = addr + chip8.registers.v[0x0] as usize,
            Op::Random(vx, val) => chip8.registers.v[vx] = val & fastrand::u8(..),
            Op::Draw(vx, vy, row) => {
                let x_coord = (chip8.registers.v[vx] % 64) as usize;
                let y_coord = (chip8.registers.v[vy] % 32) as usize;
                chip8.registers.v[15] = 0;
                let sprite_addr = chip8.registers.i;
                let bitmask: u8 = 0b1000_0000;

                for i in 0..row {
                    let sprite_byte = chip8.memory.get_byte(sprite_addr + i);
                    for j in 0..8 {
                        if x_coord + j > 64 {
                            break;
                        }
                        let display_index = 64 * (y_coord + i) + x_coord + j;
                        let current_pixel = chip8.display[display_index];
                        // gets the value of a single bit from the row
                        let sprite_pixel: bool = ((bitmask >> j) & sprite_byte).count_ones() > 0;

                        if current_pixel & sprite_pixel {
                            chip8.display[display_index] = false;
                            chip8.registers.v[15] = 1;
                        } else {
                            chip8.display[display_index] = sprite_pixel;
                        }
                    }
                    if y_coord + i > 32 {
                        break;
                    }
                }
            }
            Op::SkipIfKeyDown(vx) => {
                if chip8.keypad[vx] {
                    chip8.registers.pc += 2
                }
            }
            Op::SkipIfKeyUp(vx) => {
                if !chip8.keypad[vx] {
                    chip8.registers.pc += 2
                }
            }
            Op::GetDelayTimer(vx) => chip8.registers.v[vx] = chip8.registers.timer_delay,
            Op::SetDelayTimer(vx) => chip8.registers.timer_delay = chip8.registers.v[vx],
            Op::SetSoundTimer(vx) => chip8.registers.timer_sound = chip8.registers.v[vx],
            Op::AddToIndex(vx) => {
                let x = chip8.registers.v[vx] as usize;
                let sum = x + chip8.registers.i;
                // the valid address range (0x000 - 0xFFF)
                if sum > 0xFFF {
                    chip8.registers.v[0xF] = 1;
                }
                chip8.registers.i = sum;
            }
            Op::GetKey(vx) => {
                chip8.registers.pc -= 2;
                for (i, key) in chip8.keypad.iter().enumerate() {
                    if *key {
                        chip8.registers.pc += 2;
                        chip8.registers.v[vx] = i as u8;
                        break;
                    }
                }
            }
            Op::FontChar(vx) => {
                // The character is stored only in the last nibble of VX
                let char_addr: usize = (0x0F & chip8.registers.v[vx]).into();
                chip8.registers.i = FONTSET_START_ADDR + char_addr;
            }
            Op::BCDConv(vx) => {
                let addr = chip8.registers.i;
                let mut x = chip8.registers.v[vx];

                let mut digits: Vec<u8> = Vec::new();
                digits.push(x / 100);
                x %= 100;
                digits.push(x / 10);
                x %= 10;
                digits.push(x);

                chip8.memory.bytes[addr..(3 + addr)].copy_from_slice(&digits[..3]);
            }
            Op::StoreMem(vx) => {
                let addr = chip8.registers.i;
                for i in 0..=vx {
                    chip8.memory.bytes[addr + i] = chip8.registers.v[i];
                }
            }
            Op::LoadMem(vx) => {
                let addr = chip8.registers.i;
                for i in 0..=vx {
                    chip8.registers.v[i] = chip8.memory.bytes[addr + i];
                }
            }
            Op::CallSubroutine(addr) => {
                chip8.memory.stack.push(chip8.registers.pc);
                chip8.registers.pc = addr;
            }
            Op::RetSubroutine => chip8.registers.pc = chip8.memory.stack.pop().unwrap(),
        }
    }
}
