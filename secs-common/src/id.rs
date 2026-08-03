use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SystemByte(pub u32);

impl SystemByte {
    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1).max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemByteSource {
    next: u32,
    min: u32,
    max: u32,
}

impl SystemByteSource {
    pub const fn new() -> Self {
        Self::with_range(1, 1, u32::MAX)
    }

    pub const fn with_range(start: u32, min: u32, max: u32) -> Self {
        Self {
            next: start,
            min,
            max,
        }
    }

    pub fn current(&self) -> SystemByte {
        SystemByte(self.next)
    }

    fn advance(&mut self) {
        let next = self.next.wrapping_add(1);
        self.next = if next < self.min || next > self.max {
            self.min
        } else {
            next
        };
    }

    pub fn next_system_byte(&mut self) -> SystemByte {
        let current = SystemByte(self.next);
        self.advance();
        current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub u16);

impl SessionId {
    pub const CONTROL: Self = Self(0xFFFF);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_byte_source_returns_current_then_advances() {
        let mut source = SystemByteSource::with_range(10, 10, 12);

        assert_eq!(source.next_system_byte(), SystemByte(10));
        assert_eq!(source.next_system_byte(), SystemByte(11));
        assert_eq!(source.next_system_byte(), SystemByte(12));
    }

    #[test]
    fn test_system_byte_source_wraps_to_min() {
        let mut source = SystemByteSource::with_range(10, 10, 11);

        assert_eq!(source.next_system_byte(), SystemByte(10));
        assert_eq!(source.next_system_byte(), SystemByte(11));
        assert_eq!(source.next_system_byte(), SystemByte(10));
    }

    #[test]
    fn test_system_byte_source_uses_full_range_by_default() {
        let mut source = SystemByteSource::new();

        assert_eq!(source.next_system_byte(), SystemByte(1));
        assert_eq!(source.next_system_byte(), SystemByte(2));
    }
}
