use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Guid(
    pub u32,
    pub u16,
    pub u16,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
);

impl From<[u8; 16]> for Guid {
    fn from(bytes: [u8; 16]) -> Self {
        Guid(
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_le_bytes([bytes[4], bytes[5]]),
            u16::from_le_bytes([bytes[6], bytes[7]]),
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        )
    }
}

impl From<Guid> for [u8; 16] {
    fn from(guid: Guid) -> Self {
        [
            guid.0.to_le_bytes()[0],
            guid.0.to_le_bytes()[1],
            guid.0.to_le_bytes()[2],
            guid.0.to_le_bytes()[3],
            guid.1.to_le_bytes()[0],
            guid.1.to_le_bytes()[1],
            guid.2.to_le_bytes()[0],
            guid.2.to_le_bytes()[1],
            guid.3,
            guid.4,
            guid.5,
            guid.6,
            guid.7,
            guid.8,
            guid.9,
            guid.10,
        ]
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, self.10,
        )
    }
}
