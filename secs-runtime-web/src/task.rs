use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use secs_runtime_core::{TaskFuture, TaskRunner, TaskSpawnError};

pub struct WebTaskRunner<T> {
    completed: Rc<RefCell<VecDeque<T>>>,
}

impl<T> WebTaskRunner<T> {
    pub fn new() -> Self {
        Self {
            completed: Rc::new(RefCell::new(VecDeque::new())),
        }
    }
}

impl<T> Default for WebTaskRunner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> TaskRunner<T> for WebTaskRunner<T> {
    fn spawn_boxed(&mut self, future: TaskFuture<T>) -> Result<(), TaskSpawnError> {
        let completed = self.completed.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = future.await;
            completed.borrow_mut().push_back(result);
        });

        Ok(())
    }

    fn poll_completed(&mut self) -> VecDeque<T> {
        std::mem::take(&mut self.completed.borrow_mut())
    }
}
