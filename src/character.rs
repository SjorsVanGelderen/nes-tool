pub struct Character {
    pub bytes: [u8; 8192],
    // pub sampler: [u8; 8192],
}

impl Character {
    pub fn zero() -> Character {
        Character {
            bytes: [0; 8192],
            // sampler: [],
        }
    }
}