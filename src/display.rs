use core::{
    cell::Cell,
    future::{join, Future},
    task::{Poll, Waker},
};

use embedded_hal_async::delay::DelayNs;
use futures::{pin_mut, select_biased, FutureExt};
use pin_project::pin_project;

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn yield_now() -> impl Future<Output = ()> {
    YieldNow(false)
}

struct TimerCancelled;

struct TimerCanceller<'a> {
    cancelled_flag: &'a Cell<bool>,
    waker: &'a Cell<Option<Waker>>,
}

impl<'a> TimerCanceller<'a> {
    fn cancel(&self) {
        self.cancelled_flag.set(true);
        if let Some(waker) = self.waker.take() {
            waker.clone().wake();
            self.waker.set(Some(waker));
        }
    }
}

struct CancellableTimerHandle<'a, D> {
    delay: &'a mut D,
    cancelled_flag: &'a Cell<bool>,
    waker: &'a Cell<Option<Waker>>,
}

struct CancellableTimer<D> {
    delay: D,
    cancelled_flag: Cell<bool>,
    waker: Cell<Option<Waker>>,
}

impl<D> CancellableTimer<D> {
    fn new(delay: D) -> Self {
        Self {
            delay,
            cancelled_flag: Cell::new(false),
            waker: Cell::new(None),
        }
    }

    fn split<'a>(&'a mut self) -> (TimerCanceller<'a>, CancellableTimerHandle<'a, D>) {
        (
            TimerCanceller {
                cancelled_flag: &self.cancelled_flag,
                waker: &self.waker,
            },
            CancellableTimerHandle {
                delay: &mut self.delay,
                cancelled_flag: &self.cancelled_flag,
                waker: &self.waker,
            },
        )
    }
}

#[pin_project]
struct CancellableTimerHandleFuture<'a, F> {
    flag: &'a Cell<bool>,
    waker: &'a Cell<Option<Waker>>,
    #[pin]
    delay_future: F,
}

impl<'a, F: Future<Output = ()>> Future for CancellableTimerHandleFuture<'a, F> {
    type Output = Result<(), TimerCancelled>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.waker.set(Some(cx.waker().clone()));
        if self.flag.get() {
            return Poll::Ready(Err(TimerCancelled));
        }

        if let Poll::Ready(()) = self.project().delay_future.poll(cx) {
            return Poll::Ready(Ok(()));
        }

        return Poll::Pending;
    }
}

impl<'a, D> CancellableTimerHandle<'a, D> {
    fn wait(&mut self, ns: u32) -> impl Future<Output = Result<(), TimerCancelled>> + '_
    where
        D: DelayNs,
    {
        return CancellableTimerHandleFuture {
            flag: &self.cancelled_flag,
            delay_future: self.delay.delay_ns(ns),
            waker: &self.waker,
        };
    }
}

#[derive(Clone, Copy, PartialEq)]
enum AnimationTask {
    Task1,
    Task2,
    Task3,
}

struct ControlState {
    task: Cell<AnimationTask>,
    ns: Cell<u32>,
}

struct Driver<D> {
    cs: ControlState,
    timer: CancellableTimer<D>,
}

struct Controller<'a>(&'a ControlState);

async fn task1<'a, D: DelayNs>(
    timer: &mut CancellableTimerHandle<'a, D>,
) -> Result<!, TimerCancelled> {
    loop {
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
    }
}

async fn task2<'a, D: DelayNs>(
    timer: &mut CancellableTimerHandle<'a, D>,
) -> Result<!, TimerCancelled> {
    loop {
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
    }
}

async fn task3<'a, D: DelayNs>(
    timer: &mut CancellableTimerHandle<'a, D>,
) -> Result<!, TimerCancelled> {
    loop {
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
        timer.wait(100).await?;
    }
}

impl<D: DelayNs> Driver<D> {
    pub fn split<'a>(&'a mut self) -> (Controller<'a>, impl Future<Output = !> + 'a) {
        (Controller(&self.cs), async {
            let (canceller, mut timer) = self.timer.split();
            let fut1 = async {
                let mut task = None;
                loop {
                    let new_task = self.cs.task.get();
                    if Some(new_task) != task {
                        task = Some(new_task);
                        canceller.cancel();
                    }
                    yield_now().await;
                }
            }
            .fuse();
            let fut2 = async {
                loop {
                    match self.cs.task.get() {
                        AnimationTask::Task1 => {
                            let _ = task1(&mut timer).await;
                        }
                        AnimationTask::Task2 => {
                            let _ = task2(&mut timer).await;
                        }
                        AnimationTask::Task3 => {
                            let _ = task3(&mut timer).await;
                        }
                    }
                }
            }
            .fuse();
            pin_mut!(fut1, fut2);
            loop {
                // fut2 should be polled first due to fut1 using yield_now
                // which would lead to thread starvation
                select_biased! {
                    _ = fut2 => {},
                    _ = fut1 => {},
                }
            }
        })
    }
}
