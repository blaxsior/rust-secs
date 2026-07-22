use crate::id::SystemByte;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionOwner {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransferContext {
    Send,
    Recv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionKey {
    pub owner: TransactionOwner,
    pub system_byte: SystemByte,
}

impl TransactionKey {
    pub fn next(&self) -> Self {
        Self {
            owner: self.owner,
            system_byte: self.system_byte.next(),
        }
    }

    pub fn new(owner: TransactionOwner, system_byte: SystemByte) -> Self {
        Self { owner, system_byte }
    }

    pub fn from(context: TransferContext, is_primary: bool, system_byte: SystemByte) -> Self {
        let owner = match (context, is_primary) {
            (TransferContext::Send, true) => TransactionOwner::Local,
            (TransferContext::Recv, false) => TransactionOwner::Local,
            _ => TransactionOwner::Remote,
        };

        Self::new(owner, system_byte)
    }
}
