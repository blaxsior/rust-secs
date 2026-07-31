use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
};
use core::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecsKey(String);

impl SecsKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecsKey").field(&self.0).finish()
    }
}

impl From<String> for SecsKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecsKey {
    fn from(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl From<u32> for SecsKey {
    fn from(value: u32) -> Self {
        Self::new(value.to_string())
    }
}
