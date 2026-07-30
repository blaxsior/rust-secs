use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

pub type TaskFuture<T> = Pin<Box<dyn Future<Output = T>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskSpawnError;

pub trait TaskRunner<T> {
    fn spawn_boxed(&mut self, future: TaskFuture<T>) -> Result<(), TaskSpawnError>;

    fn poll_completed(&mut self) -> VecDeque<T>;
}

pub struct TaskQueue<T> {
    tasks: BTreeMap<TaskId, TaskFuture<T>>,
    ready: ReadyQueue,
    next_task_id: u64,
}

impl<T> TaskQueue<T> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready: ReadyQueue::new(),
            next_task_id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    fn spawn_task(&mut self, future: TaskFuture<T>) -> Result<TaskId, TaskSpawnError> {
        let task_id = self.next_task_id()?;
        self.tasks.insert(task_id, future);
        self.ready.push(task_id);
        Ok(task_id)
    }

    fn poll_completed(&mut self) -> VecDeque<T> {
        let mut completed = VecDeque::new();

        while let Some(task_id) = self.ready.pop() {
            let Some(task) = self.tasks.get_mut(&task_id) else {
                continue;
            };

            let waker = task_waker(task_id, self.ready.clone());
            let mut cx = Context::from_waker(&waker);

            if let Poll::Ready(output) = task.as_mut().poll(&mut cx) {
                self.tasks.remove(&task_id);
                completed.push_back(output);
            }
        }

        completed
    }

    fn next_task_id(&mut self) -> Result<TaskId, TaskSpawnError> {
        let start = self.next_task_id;

        loop {
            let task_id = TaskId(self.next_task_id);
            self.next_task_id = self.next_task_id.wrapping_add(1);

            if !self.tasks.contains_key(&task_id) {
                return Ok(task_id);
            }

            if self.next_task_id == start {
                return Err(TaskSpawnError);
            }
        }
    }
}

impl<T> Default for TaskQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TaskRunner<T> for TaskQueue<T> {
    fn spawn_boxed(&mut self, future: TaskFuture<T>) -> Result<(), TaskSpawnError> {
        self.spawn_task(future).map(|_| ())
    }

    fn poll_completed(&mut self) -> VecDeque<T> {
        TaskQueue::poll_completed(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

#[derive(Clone)]
struct ReadyQueue {
    inner: Rc<RefCell<VecDeque<TaskId>>>,
}

impl ReadyQueue {
    fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    fn push(&self, task_id: TaskId) {
        self.inner.borrow_mut().push_back(task_id);
    }

    fn pop(&self) -> Option<TaskId> {
        self.inner.borrow_mut().pop_front()
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}

fn task_waker(task_id: TaskId, ready: ReadyQueue) -> Waker {
    let wake_data = Rc::new(TaskWake { task_id, ready });
    unsafe { Waker::from_raw(raw_waker(wake_data)) }
}

struct TaskWake {
    task_id: TaskId,
    ready: ReadyQueue,
}

fn raw_waker(wake_data: Rc<TaskWake>) -> RawWaker {
    RawWaker::new(Rc::into_raw(wake_data).cast(), &VTABLE)
}

unsafe fn clone_waker(data: *const ()) -> RawWaker {
    let wake_data = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    let cloned = wake_data.clone();
    let _ = Rc::into_raw(wake_data);
    raw_waker(cloned)
}

unsafe fn wake(data: *const ()) {
    let wake_data = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    wake_data.ready.push(wake_data.task_id);
}

unsafe fn wake_by_ref(data: *const ()) {
    let wake_data = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
    wake_data.ready.push(wake_data.task_id);
    let _ = Rc::into_raw(wake_data);
}

unsafe fn drop_waker(data: *const ()) {
    let _ = unsafe { Rc::<TaskWake>::from_raw(data.cast()) };
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promise;

    #[test]
    fn task_waker_pushes_task_to_ready_queue() {
        let ready = ReadyQueue::new();
        let waker = task_waker(TaskId(5), ready.clone());

        assert!(ready.is_empty());
        waker.wake_by_ref();
        assert_eq!(ready.pop(), Some(TaskId(5)));

        waker.wake();
        assert_eq!(ready.pop(), Some(TaskId(5)));
    }

    #[test]
    fn task_queue_polls_spawned_ready_task() {
        let mut tasks = TaskQueue::new();
        tasks.spawn_boxed(Box::pin(async { 10 })).unwrap();

        let mut completed = tasks.poll_completed();

        assert_eq!(completed.pop_front(), Some(10));
        assert!(completed.is_empty());
        assert!(tasks.is_empty());
    }

    #[test]
    fn task_queue_repolls_task_when_promise_resolves() {
        let mut tasks = TaskQueue::new();
        let (resolver, future) = promise::<u8, ()>();
        tasks
            .spawn_boxed(Box::pin(async move { future.await.unwrap() }))
            .unwrap();

        assert!(tasks.poll_completed().is_empty());
        assert_eq!(tasks.len(), 1);

        resolver.resolve(33);

        let mut completed = tasks.poll_completed();
        assert_eq!(completed.pop_front(), Some(33));
        assert!(tasks.is_empty());
    }
}
