// 16 8-bit registers called V0-VF,
// and an 16-bit instruction register called I
// for storing addresses (12-bit for 0x000-0xFFF)
pub struct Registers {
    pub v: [u8; 16],
    pub i: usize, // 16-bit index register
}

impl Registers {
    pub fn new() -> Self {
        Self { v: [0; 16], i: 0 }
    }
}
