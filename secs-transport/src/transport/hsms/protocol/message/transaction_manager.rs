use crate::transport::TransactionKey;
use crate::transport::hsms::{HsmsHeader};
use crate::transport::hsms::protocol::message::transaction::HsmsMessageTransaction;
use alloc::collections::BTreeMap;

pub struct HsmsTransactionManager {
    transaction_map: BTreeMap<TransactionKey, HsmsMessageTransaction>,
}

impl HsmsTransactionManager {
    pub fn new() -> Self {
        Self {
            transaction_map: BTreeMap::new(),
        }
    }

    pub fn create_send(&mut self, key: &TransactionKey, header: HsmsHeader) -> Option<&mut HsmsMessageTransaction> {
        let transaction = HsmsMessageTransaction::new_send(*key, header);
        self.transaction_map.insert(*key, transaction);

        self.find(key)
    }

    pub fn create_recv(&mut self, key: &TransactionKey) -> Option<&mut HsmsMessageTransaction> {
        let transaction = HsmsMessageTransaction::new_recv(*key);
        self.transaction_map.insert(*key, transaction);

        self.find(key)
    }

    /// 대상 트랜잭션을 찾아 반환한다.
    pub fn find(&mut self, key: &TransactionKey) -> Option<&mut HsmsMessageTransaction> {
        return self.transaction_map.get_mut(key);
    }

    /// 대상 트랜잭션을 제거한다.
    pub fn remove(&mut self, key: &TransactionKey) {
        self.transaction_map.remove(key);
    }
}
