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
    use alloc::rc::Rc;
    use core::{
        cell::RefCell,
        task::{Context, RawWaker, RawWakerVTable},
    };

    #[test]
    fn pending_promise_stores_waker_and_wakes_on_resolve() {
        let (resolver, mut future) = promise::<u8, ()>();
        let wake_count = Rc::new(RefCell::new(0));
        let waker = test_waker(wake_count.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(Pin::new(&mut future).poll(&mut cx), Poll::Pending));
        assert_eq!(*wake_count.borrow(), 0);

        assert!(resolver.resolve(42));

        assert_eq!(*wake_count.borrow(), 1);
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(42))
        ));
    }

    #[test]
    fn resolved_promise_is_ready_without_waiting() {
        let (resolver, mut future) = promise::<u8, ()>();
        let wake_count = Rc::new(RefCell::new(0));
        let waker = test_waker(wake_count.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(resolver.resolve(11));

        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(11))
        ));
        assert_eq!(*wake_count.borrow(), 0);
    }

    #[test]
    fn promise_can_only_complete_once() {
        let (resolver, mut future) = promise::<u8, &'static str>();
        let wake_count = Rc::new(RefCell::new(0));
        let waker = test_waker(wake_count.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(Pin::new(&mut future).poll(&mut cx), Poll::Pending));

        assert!(resolver.resolve(1));
        assert!(!resolver.resolve(2));
        assert!(!resolver.reject("late error"));

        assert_eq!(*wake_count.borrow(), 1);
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Ok(1))
        ));
    }

    fn test_waker(wake_count: Rc<RefCell<u8>>) -> Waker {
        unsafe { Waker::from_raw(raw_waker(wake_count)) }
    }

    fn raw_waker(wake_count: Rc<RefCell<u8>>) -> RawWaker {
        RawWaker::new(Rc::into_raw(wake_count).cast(), &VTABLE)
    }

    unsafe fn clone_waker(data: *const ()) -> RawWaker {
        let wake_count = unsafe { Rc::<RefCell<u8>>::from_raw(data.cast()) };
        let cloned = wake_count.clone();
        let _ = Rc::into_raw(wake_count);
        raw_waker(cloned)
    }

    unsafe fn wake(data: *const ()) {
        let wake_count = unsafe { Rc::<RefCell<u8>>::from_raw(data.cast()) };
        *wake_count.borrow_mut() += 1;
    }

    unsafe fn wake_by_ref(data: *const ()) {
        let wake_count = unsafe { Rc::<RefCell<u8>>::from_raw(data.cast()) };
        *wake_count.borrow_mut() += 1;
        let _ = Rc::into_raw(wake_count);
    }

    unsafe fn drop_waker(data: *const ()) {
        let _ = unsafe { Rc::<RefCell<u8>>::from_raw(data.cast()) };
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);
}
