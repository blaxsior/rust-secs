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

use crate::promise::{PromiseFuture, PromiseResolver, promise};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

#[derive(Clone)]
pub struct ReadyQueue {
    inner: Rc<RefCell<VecDeque<TaskId>>>,
}

impl ReadyQueue {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn push(&self, task_id: TaskId) {
        self.inner.borrow_mut().push_back(task_id);
    }

    pub fn pop(&self) -> Option<TaskId> {
        self.inner.borrow_mut().pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}

impl Default for ReadyQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub type TaskFuture<T> = Pin<Box<dyn Future<Output = T>>>;

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

    pub fn spawn<F>(&mut self, future: F) -> TaskId
    where
        F: Future<Output = T> + 'static,
    {
        let task_id = self.next_task_id();
        self.tasks.insert(task_id, Box::pin(future));
        self.ready.push(task_id);
        task_id
    }

    pub fn spawn_boxed(&mut self, future: TaskFuture<T>) -> TaskId {
        let task_id = self.next_task_id();
        self.tasks.insert(task_id, future);
        self.ready.push(task_id);
        task_id
    }

    pub fn spawn_promise<V, E, F, Fut>(&mut self, build: F) -> PromiseResolver<V, E>
    where
        F: FnOnce(PromiseFuture<V, E>) -> Fut,
        Fut: Future<Output = T> + 'static,
        V: 'static,
        E: 'static,
    {
        self.spawn_promise_with_id(build).1
    }

    pub fn spawn_promise_with_id<V, E, F, Fut>(
        &mut self,
        build: F,
    ) -> (TaskId, PromiseResolver<V, E>)
    where
        F: FnOnce(PromiseFuture<V, E>) -> Fut,
        Fut: Future<Output = T> + 'static,
        V: 'static,
        E: 'static,
    {
        let (resolver, future) = promise::<V, E>();
        let task_id = self.spawn(build(future));
        (task_id, resolver)
    }

    pub fn poll_completed(&mut self) -> VecDeque<T> {
        self.poll_ready()
            .into_iter()
            .map(|(_, output)| output)
            .collect()
    }

    pub fn poll_ready(&mut self) -> VecDeque<(TaskId, T)> {
        let mut completed = VecDeque::new();

        while let Some(task_id) = self.ready.pop() {
            let Some(task) = self.tasks.get_mut(&task_id) else {
                continue;
            };

            let waker = task_waker(task_id, self.ready.clone());
            let mut cx = Context::from_waker(&waker);

            if let Poll::Ready(output) = task.as_mut().poll(&mut cx) {
                self.tasks.remove(&task_id);
                completed.push_back((task_id, output));
            }
        }

        completed
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn ready_queue(&self) -> ReadyQueue {
        self.ready.clone()
    }

    fn next_task_id(&mut self) -> TaskId {
        let task_id = TaskId(self.next_task_id);
        self.next_task_id += 1;
        task_id
    }
}

impl<T> Default for TaskQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn task_waker(task_id: TaskId, ready: ReadyQueue) -> Waker {
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
        let task_id = tasks.spawn(async { 10 });

        let mut completed = tasks.poll_ready();

        assert_eq!(completed.pop_front(), Some((task_id, 10)));
        assert!(completed.is_empty());
        assert!(tasks.is_empty());
    }

    #[test]
    fn task_queue_repolls_task_when_promise_resolves() {
        let mut tasks = TaskQueue::new();
        let resolver =
            tasks.spawn_promise::<u8, (), _, _>(|future| async move { future.await.unwrap() });

        assert!(tasks.poll_ready().is_empty());
        assert_eq!(tasks.len(), 1);

        resolver.resolve(33);

        let mut completed = tasks.poll_ready();
        assert_eq!(completed.pop_front().map(|(_, output)| output), Some(33));
        assert!(tasks.is_empty());
    }

    #[test]
    fn promise_queue_usage_example() {
        let mut tasks = TaskQueue::new();

        let resolver =
            tasks.spawn_promise::<&'static str, &'static str, _, _>(|response| async move {
                let value = response.await?;
                Ok::<_, &'static str>(value.len())
            });

        let completed = tasks.poll_ready();
        assert!(completed.is_empty());
        assert_eq!(tasks.len(), 1);

        resolver.resolve("selected");

        let mut completed = tasks.poll_completed();
        assert_eq!(completed.pop_front(), Some(Ok(8)));
        assert!(completed.is_empty());
        assert!(tasks.is_empty());
    }

    #[test]
    fn promise_queue_reject_example() {
        let mut tasks = TaskQueue::new();

        let resolver = tasks.spawn_promise::<u8, &'static str, _, _>(|response| async move {
            let value = response.await?;
            Ok::<_, &'static str>(value + 1)
        });

        assert!(tasks.poll_ready().is_empty());

        resolver.reject("timeout");

        let mut completed = tasks.poll_completed();
        assert_eq!(completed.pop_front(), Some(Err("timeout")));
        assert!(completed.is_empty());
        assert!(tasks.is_empty());
    }
}
