use alloc::rc::Rc;
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub fn promise<T, E>() -> (PromiseResolver<T, E>, PromiseFuture<T, E>) {
    let slot = Rc::new(RefCell::new(PromiseState::Pending { waker: None }));

    (
        PromiseResolver { slot: slot.clone() },
        PromiseFuture { slot },
    )
}

pub struct PromiseResolver<T, E> {
    slot: Rc<RefCell<PromiseState<T, E>>>,
}

impl<T, E> PromiseResolver<T, E> {
    pub fn resolve(&self, value: T) -> bool {
        self.complete(Ok::<T, E>(value))
    }

    pub fn reject(&self, error: E) -> bool {
        self.complete(Err::<T, E>(error))
    }

    pub fn complete(&self, result: Result<T, E>) -> bool {
        let waker = {
            let mut slot = self.slot.borrow_mut();
            match core::mem::replace(&mut *slot, PromiseState::Consumed) {
                PromiseState::Pending { waker } => {
                    *slot = PromiseState::Completed {
                        result: Some(result),
                    };
                    waker
                }
                state => {
                    *slot = state;
                    return false;
                }
            }
        };

        if let Some(waker) = waker {
            waker.wake();
        }

        true
    }
}

pub struct PromiseFuture<T, E> {
    slot: Rc<RefCell<PromiseState<T, E>>>,
}

impl<T, E> Future for PromiseFuture<T, E> {
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.slot.borrow_mut();

        match &mut *slot {
            PromiseState::Pending { waker } => {
                *waker = Some(cx.waker().clone());
                Poll::Pending
            }
            PromiseState::Completed { result } => {
                let result = result
                    .take()
                    .expect("completed promise must contain result");
                *slot = PromiseState::Consumed;
                Poll::Ready(result)
            }
            PromiseState::Consumed => {
                panic!("promise polled after completion")
            }
        }
    }
}

enum PromiseState<T, E> {
    Pending { waker: Option<Waker> },
    Completed { result: Option<Result<T, E>> },
    Consumed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promise::{ReadyQueue, TaskId, task_waker};
    use core::task::Context;

    #[test]
    fn pending_promise_stores_waker_and_wakes_on_resolve() {
        let (resolver, mut future) = promise::<u8, ()>();
        let ready = ReadyQueue::new();
        let waker = task_waker(TaskId(7), ready.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(Pin::new(&mut future).poll(&mut cx), Poll::Pending));
        assert!(ready.pop().is_none());

        assert!(resolver.resolve(42));

        assert_eq!(ready.pop(), Some(TaskId(7)));
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(42))
        ));
    }

    #[test]
    fn resolved_promise_is_ready_without_waiting() {
        let (resolver, mut future) = promise::<u8, ()>();
        let ready = ReadyQueue::new();
        let waker = task_waker(TaskId(3), ready.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(resolver.resolve(11));

        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(11))
        ));
        assert!(ready.pop().is_none());
    }

    #[test]
    fn promise_can_only_complete_once() {
        let (resolver, mut future) = promise::<u8, &'static str>();
        let ready = ReadyQueue::new();
        let waker = task_waker(TaskId(9), ready.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(Pin::new(&mut future).poll(&mut cx), Poll::Pending));

        assert!(resolver.resolve(1));
        assert!(!resolver.resolve(2));
        assert!(!resolver.reject("late error"));

        assert_eq!(ready.pop(), Some(TaskId(9)));
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(1))
        ));
    }
}
