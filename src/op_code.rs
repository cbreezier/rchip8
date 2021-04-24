#[derive(Debug)]
pub struct OpCode {
    pub op: u8, // Technically a u4 but u8 makes for easier comparisons
    pub x: u8, // Technically a u4 but u8 makes for easier comparisons
    pub y: u8, // Technically a u4 but u8 makes for easier comparisons
    pub n: u8, // Technically a u4 but u8 makes for easier comparisons
    pub nn: u8,
    pub nnn: u16, // Technically a u12 but u16 makes for easier comparisons
}

impl OpCode {
    pub fn from_bytes(byte1: u8, byte2: u8) -> Self {
        Self {
            op: (byte1 >> 4) & 0xF,
            x: byte1 & 0xF,
            y: (byte2 >> 4) & 0xF,
            n: byte2 & 0xF,
            nn: byte2,
            nnn: (u16::from(byte1 & 0xF) << 8) | u16::from(byte2),
        }
    }
}
