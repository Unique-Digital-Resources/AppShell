//! `TaskPool` — a simple thread pool for concurrent sync handler execution.
//!
//! This is intentionally minimal. For production async execution, use
//! tokio's runtime or async-std. This pool allows sync handlers to run
//! on separate threads without an async runtime.

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A minimal thread pool for concurrent task execution.
pub struct TaskPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: Option<mpsc::Sender<Job>>,
}

impl TaskPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            workers.push(thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv();
                match job {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Submit a job to the pool.
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Box::new(f));
        }
    }

    /// Number of worker threads.
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        let workers = std::mem::take(&mut self.workers);
        for worker in workers {
            let _ = worker.join();
        }
    }
}

impl std::fmt::Debug for TaskPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskPool")
            .field("workers", &self.workers.len())
            .finish()
    }
}