use crate::core::SecsMessage;
use crate::transport::{
    TransactionKey, secs1::protocol::message::transaction::Secs1MessageTransaction,
};
use alloc::collections::BTreeMap;

pub struct Secs1TransactionManager {
    transaction_map: BTreeMap<TransactionKey, Secs1MessageTransaction>,
}

impl Secs1TransactionManager {
    pub fn new() -> Self {
        Self {
            transaction_map: BTreeMap::new(),
        }
    }

    pub fn create_send(
        &mut self,
        key: &TransactionKey,
        msg: SecsMessage,
    ) -> Option<&mut Secs1MessageTransaction> {
        let transaction = Secs1MessageTransaction::new_send(*key, msg);
        self.transaction_map.insert(*key, transaction);

        self.find(key)
    }

    pub fn create_recv(&mut self, key: &TransactionKey) -> Option<&mut Secs1MessageTransaction> {
        let transaction = Secs1MessageTransaction::new_recv(*key);
        self.transaction_map.insert(*key, transaction);

        self.find(key)
    }

    /// 대상 트랜잭션을 찾아 반환한다.
    pub fn find(&mut self, key: &TransactionKey) -> Option<&mut Secs1MessageTransaction> {
        return self.transaction_map.get_mut(key);
    }

    /// 대상 트랜잭션을 제거한다.
    pub fn remove(&mut self, key: &TransactionKey) {
        self.transaction_map.remove(key);
    }
}
