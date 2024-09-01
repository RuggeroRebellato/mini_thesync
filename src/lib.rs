use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::utils::Backoff;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

struct Task {
    future: Mutex<BoxFuture>,
    waker: Mutex<Option<Waker>>,
}

pub struct Runtime {
    task_sender: Sender<Arc<Task>>,
    task_receiver: Receiver<Arc<Task>>,
    stealers: Vec<Stealer<Arc<Task>>>,
}

thread_local! {
    static WORKER: RefCell<Worker<Arc<Task>>> = RefCell::new(Worker::new_fifo());
}

impl Runtime {
    pub fn new(num_threads: usize) -> Self {
        let (task_sender, task_receiver) = unbounded();
        let mut stealers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            WORKER.with(|worker| {
                stealers.push(worker.borrow().stealer());
            });
        }

        Runtime {
            task_sender,
            task_receiver,
            stealers,
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            waker: Mutex::new(None),
        });
        self.task_sender.send(task).unwrap();
    }

    pub fn run(&self) {
        crossbeam::scope(|s| {
            for _ in 0..self.stealers.len() {
                let stealers = &self.stealers;
                let task_receiver = &self.task_receiver;
                let task_sender = &self.task_sender;

                s.spawn(move |_| {
                    WORKER.with(|worker| {
                        let backoff = Backoff::new();
                        loop {
                            if let Some(task) =
                                Self::find_task(&mut worker.borrow_mut(), stealers, task_receiver)
                            {
                                Self::run_task(task, task_sender);
                                backoff.reset();
                            } else if backoff.is_completed() {
                                break;
                            } else {
                                backoff.snooze();
                            }
                        }
                    });
                });
            }
        })
        .unwrap();
    }

    fn find_task(
        local: &mut Worker<Arc<Task>>,
        stealers: &[Stealer<Arc<Task>>],
        receiver: &Receiver<Arc<Task>>,
    ) -> Option<Arc<Task>> {
        local
            .pop()
            .or_else(|| {
                stealers
                    .iter()
                    .filter_map(|stealer| match stealer.steal() {
                        Steal::Success(task) => Some(task),
                        _ => None,
                    })
                    .next()
            })
            .or_else(|| receiver.try_recv().ok())
    }

    fn run_task(task: Arc<Task>, sender: &Sender<Arc<Task>>) {
        let waker = TaskWaker {
            task: task.clone(),
            sender: sender.clone(),
        }
        .into_waker();
        let mut context = Context::from_waker(&waker);

        let mut should_reschedule = false;

        {
            if let Ok(mut future) = task.future.lock() {
                if future.as_mut().poll(&mut context).is_pending() {
                    should_reschedule = true;
                    if let Ok(mut task_waker) = task.waker.lock() {
                        *task_waker = Some(waker);
                    }
                }
            }
        }

        if should_reschedule {
            sender.send(task).unwrap();
        }
    }
}

struct TaskWaker {
    task: Arc<Task>,
    sender: Sender<Arc<Task>>,
}

impl TaskWaker {
    fn into_waker(self) -> Waker {
        Waker::from(Arc::new(self))
    }
}

unsafe impl Send for TaskWaker {}
unsafe impl Sync for TaskWaker {}

impl std::task::Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.sender.send(self.task.clone()).unwrap();
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    std::future::poll_fn(move |_| {
        std::thread::sleep(duration);
        Poll::Ready(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[test]
    fn test_basic_task_execution() {
        let runtime = Runtime::new(1);
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        runtime.spawn(async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        runtime.run();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_tasks() {
        let runtime = Runtime::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..100 {
            let counter_clone = counter.clone();
            runtime.spawn(async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
        }

        runtime.run();

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_task_sleep() {
        let runtime = Runtime::new(1);
        let start = Instant::now();

        runtime.spawn(async {
            sleep(Duration::from_millis(100)).await;
        });

        runtime.run();

        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[test]
    fn test_concurrent_execution() {
        let runtime = Runtime::new(4);
        let start = Instant::now();

        for _ in 0..4 {
            runtime.spawn(async {
                sleep(Duration::from_millis(100)).await;
            });
        }

        runtime.run();

        // If tasks run concurrently, this should take just over 100ms, not 400ms
        assert!(start.elapsed() < Duration::from_millis(150));
    }

    #[test]
    fn test_task_completion() {
        let runtime = Runtime::new(1);
        let completed_tasks = Arc::new(Mutex::new(Vec::new()));

        for i in 0..5 {
            let completed_tasks_clone = completed_tasks.clone();
            runtime.spawn(async move {
                sleep(Duration::from_millis(50 * (5 - i as u64))).await;
                completed_tasks_clone.lock().unwrap().push(i);
            });
        }

        runtime.run();

        let final_completed_tasks = completed_tasks.lock().unwrap();
        assert_eq!(final_completed_tasks.len(), 5);
        for i in 0..5 {
            assert!(
                final_completed_tasks.contains(&i),
                "Task {} was not completed",
                i
            );
        }
    }

    #[test]
    fn test_nested_spawn() {
        let runtime = Arc::new(Runtime::new(2));
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let runtime_clone = runtime.clone();
            let counter_clone = counter.clone();
            runtime.spawn(async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                let counter_clone = counter_clone.clone();
                runtime_clone.spawn(async move {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                });
            });
        }

        runtime.run();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
