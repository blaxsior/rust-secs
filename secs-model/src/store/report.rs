use alloc::vec::Vec;

use crate::{store::StoreError, ReportId, ReportSpec};

pub trait ReportRepository {
    fn load_all(&mut self) -> Result<Vec<ReportSpec>, StoreError>;
    fn save(&mut self, spec: &ReportSpec) -> Result<(), StoreError>;
    fn remove(&mut self, id: &ReportId) -> Result<(), StoreError>;
}

/// 비어 있는 default report repository
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopReportRepository;

impl ReportRepository for NoopReportRepository {
    fn load_all(&mut self) -> Result<Vec<ReportSpec>, StoreError> {
        Ok(Vec::new())
    }

    fn save(&mut self, _spec: &ReportSpec) -> Result<(), StoreError> {
        Ok(())
    }

    fn remove(&mut self, _id: &ReportId) -> Result<(), StoreError> {
        Ok(())
    }
}
