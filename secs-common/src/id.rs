#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemByte(pub u32);

impl SystemByte {
    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1).max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(pub u16);

impl SessionId {
    pub const CONTROL: Self = Self(0xFFFF);
}
