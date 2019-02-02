pub struct PatternTable {
    pub bytes: [u8; 8192],
    pub pixels: [u8; 32768],
}

impl PatternTable {
    pub fn zero() -> PatternTable {
        PatternTable {
            bytes: [0; 8192],
            pixels: [0; 32768],
        }
    }
}