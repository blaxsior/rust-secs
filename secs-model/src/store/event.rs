use alloc::vec::Vec;

use crate::{store::StoreError, EventId, EventSpec};

pub trait EventRepository {
    fn find_all(&mut self) -> Result<Vec<EventSpec>, StoreError>;
    fn save(&mut self, spec: &EventSpec) -> Result<(), StoreError>;
    fn delete(&mut self, id: &EventId) -> Result<(), StoreError>;
}

/// 비어 있는 default event repository
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopEventRepository;

impl EventRepository for NoopEventRepository {
    fn find_all(&mut self) -> Result<Vec<EventSpec>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, _spec: &EventSpec) -> Result<(), StoreError> {
        Ok(())
    }

    fn delete(&mut self, _id: &EventId) -> Result<(), StoreError> {
        Ok(())
    }
}
