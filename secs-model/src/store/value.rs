use alloc::vec::Vec;

use crate::{store::StoreError, ValueData, ValueId, ValueSpec};

/// value spec을 읽어오는 repository
/// Value에 대한 타입 정보 등은 런타임에 변경되어서는 안되는 정보로, save / remove 등을 구현하지 않을 예정
pub trait ValueSpecRepository {
    fn find_all(&mut self) -> Result<Vec<ValueSpec>, StoreError>;
    // fn save(&mut self, spec: &ValueSpec) -> Result<(), StoreError>;
    // fn remove(&mut self, id: &ValueId) -> Result<(), StoreError>;
}

/// Value에 대한 실제 값을 저장하는 repository
pub trait ValueDataRepository {
    fn find_all(&mut self) -> Result<Vec<ValueData>, StoreError>;
    fn save(&mut self, data: &ValueData) -> Result<(), StoreError>;
    fn delete(&mut self, id: &ValueId) -> Result<(), StoreError>;
}

/// 비어 있는 default value spec repository
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopValueSpecRepository;

impl ValueSpecRepository for NoopValueSpecRepository {
    fn find_all(&mut self) -> Result<Vec<ValueSpec>, StoreError> {
        Ok(Vec::new())
    }
}

/// 비어 있는 default value data repository
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopValueDataRepository;

impl ValueDataRepository for NoopValueDataRepository {
    fn find_all(&mut self) -> Result<Vec<ValueData>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, _data: &ValueData) -> Result<(), StoreError> {
        Ok(())
    }

    fn delete(&mut self, _id: &ValueId) -> Result<(), StoreError> {
        Ok(())
    }
}
