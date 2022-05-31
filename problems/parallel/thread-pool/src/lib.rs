#![forbid(unsafe_code)]

use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    thread,
};

////////////////////////////////////////////////////////////////////////////////

pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        let mut workers = Vec::with_capacity(thread_count);
        let (sender, r) = unbounded();

        for _ in 0..thread_count {
            let r: Receiver<Box<dyn FnOnce() + Send>> = r.clone();
            workers.push(thread::spawn(move || {
                while let Ok(routine) = r.recv() {
                    routine();
                }
            }));
        }
        Self { workers, sender }
    }

    pub fn spawn<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (s, receiver) = unbounded();

        self.sender
            .send(Box::new(move || {
                match catch_unwind(AssertUnwindSafe(task)) {
                    Ok(val) => s.send(Ok(val)).ok(),
                    Err(_) => s.send(Err(JoinError {})).ok(),
                };
            }))
            .unwrap();

        JoinHandle { receiver }
    }

    pub fn shutdown(self) {
        drop(self.sender);
        for worker in self.workers.into_iter() {
            worker.join().unwrap();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct JoinHandle<T> {
    receiver: Receiver<Result<T, JoinError>>,
}

#[derive(Debug)]
pub struct JoinError {}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, JoinError> {
        self.receiver.recv().unwrap()
    }
}
