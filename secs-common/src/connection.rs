#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionRole {
    Active,
    Passive,
}

impl ConnectionRole {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Passive)
    }
}
