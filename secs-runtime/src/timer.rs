use secs_runtime_core::SecsTimeoutUnit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutConfig<D> {
    pub t1: D,
    pub t2: D,
    pub t3: D,
    pub t4: D,
    pub t5: D,
    pub t6: D,
    pub t7: D,
    pub t8: D,
}

impl<D: Copy> TimeoutConfig<D> {
    pub(crate) fn duration_for(&self, unit: SecsTimeoutUnit) -> D {
        match unit {
            SecsTimeoutUnit::T1 => self.t1,
            SecsTimeoutUnit::T2 => self.t2,
            SecsTimeoutUnit::T3(_) => self.t3,
            SecsTimeoutUnit::T4(_) => self.t4,
            SecsTimeoutUnit::T5 => self.t5,
            SecsTimeoutUnit::T6 => self.t6,
            SecsTimeoutUnit::T7 => self.t7,
            SecsTimeoutUnit::T8 => self.t8,
        }
    }
}
