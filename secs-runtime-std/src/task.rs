use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use futures::{
    executor::LocalSpawner,
    task::{LocalSpawnExt, SpawnError},
};
use secs_runtime_core::{TaskFuture, TaskRunner, TaskSpawnError};

pub struct LocalPoolTaskRunner<T> {
    spawner: LocalSpawner,
    completed: Rc<RefCell<VecDeque<T>>>,
}

impl<T> LocalPoolTaskRunner<T> {
    pub fn new(spawner: LocalSpawner) -> Self {
        Self {
            spawner,
            completed: Rc::new(RefCell::new(VecDeque::new())),
        }
    }
}

impl<T: 'static> TaskRunner<T> for LocalPoolTaskRunner<T> {
    fn spawn_boxed(&mut self, future: TaskFuture<T>) -> Result<(), TaskSpawnError> {
        let completed = self.completed.clone();
        self.spawner
            .spawn_local(async move {
                let result = future.await;
                completed.borrow_mut().push_back(result);
            })
            .map_err(map_spawn_error)
    }

    fn poll_completed(&mut self) -> VecDeque<T> {
        std::mem::take(&mut self.completed.borrow_mut())
    }
}

fn map_spawn_error(_: SpawnError) -> TaskSpawnError {
    TaskSpawnError
}
