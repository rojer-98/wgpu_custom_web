use bitflags::bitflags;

#[derive(Debug)]
pub struct ObjectMetadata {
    pub id: usize,
    pub coords: (u32, u32),
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ControlFlags : u32 {
        const Click = 0b00000001;
        const Convert = 0b00000010;
    }
}

impl ControlFlags {
    pub fn to_u32(&self) -> u32 {
        self.bits()
    }

    #[warn(dead_code)]
    pub fn to_u64(&self) -> u64 {
        self.bits() as u64
    }
}
