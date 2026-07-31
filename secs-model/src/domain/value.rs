use secs_ii::item::{Secs2FormatCode, Secs2Variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueBinding {
    Runtime,
    Persistent,
    Computed,
    External,
}

impl ValueBinding {
    pub fn is_persistent(self) -> bool {
        matches!(self, Self::Persistent)
    }
}

#[derive(Debug)]
pub struct ValueEntry {
    format: Secs2FormatCode,
    binding: ValueBinding,
    writable: bool,
    value: Option<Secs2Variant>,
}

impl ValueEntry {
    pub fn new(format: Secs2FormatCode, binding: ValueBinding) -> Self {
        Self {
            format,
            binding,
            writable: true,
            value: None,
        }
    }

    pub fn with_value(mut self, value: Secs2Variant) -> Self {
        self.value = Some(value);
        self
    }

    pub fn readonly(mut self) -> Self {
        self.writable = false;
        self
    }

    pub fn format(&self) -> Secs2FormatCode {
        self.format
    }

    pub fn binding(&self) -> ValueBinding {
        self.binding
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    pub fn value(&self) -> Option<&Secs2Variant> {
        self.value.as_ref()
    }

    pub fn set_value(&mut self, value: Secs2Variant) {
        self.value = Some(value);
    }
}
