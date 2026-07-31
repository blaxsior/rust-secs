use alloc::vec::Vec;

use crate::SecsKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportEntry {
    variables: Vec<SecsKey>,
}

impl ReportEntry {
    pub fn new(variables: Vec<SecsKey>) -> Self {
        Self { variables }
    }

    pub fn variables(&self) -> &[SecsKey] {
        &self.variables
    }

    pub fn variables_mut(&mut self) -> &mut Vec<SecsKey> {
        &mut self.variables
    }
}
