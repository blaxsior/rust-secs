use alloc::vec::Vec;

use crate::{store::StoreError, ReportId, ReportSpec};

pub trait ReportRepository {
    fn find_all(&mut self) -> Result<Vec<ReportSpec>, StoreError>;
    fn save(&mut self, spec: &ReportSpec) -> Result<(), StoreError>;
    fn delete(&mut self, id: &ReportId) -> Result<(), StoreError>;
}

/// 비어 있는 default report repository
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopReportRepository;

impl ReportRepository for NoopReportRepository {
    fn find_all(&mut self) -> Result<Vec<ReportSpec>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, _spec: &ReportSpec) -> Result<(), StoreError> {
        Ok(())
    }

    fn delete(&mut self, _id: &ReportId) -> Result<(), StoreError> {
        Ok(())
    }
}
