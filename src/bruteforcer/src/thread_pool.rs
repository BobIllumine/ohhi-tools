//! Generic fan-out/fan-in worker pool.
//!
//! `ThreadPool<I, O>` distributes work items across `N` worker threads, each
//! running `Fn(&I) -> O`. Workers loop on a shared `Arc<Mutex<Receiver<I>>>`
//! (the lock makes fan-out safe — `mpsc::Receiver` isn't `Sync` on its own).
//! Results are collected via a separate `mpsc` channel.
//!
//! Shutdown: `collect()` drops the work sender, which signals the channel
//! closed. Workers exit their loop on the next `recv()` error, then `collect`
//! joins all handles.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    work_tx: mpsc::Sender<I>,
    // Arc<Mutex<Receiver>> makes the receiver shareable across threads.
    // mpsc::Receiver is not Sync; the Mutex provides the required exclusion so
    // only one worker blocks on recv() at a time.
    work_rx: Arc<Mutex<mpsc::Receiver<I>>>,
    res_tx: mpsc::Sender<(I, O)>,
    res_rx: Arc<Mutex<mpsc::Receiver<(I, O)>>>,
    num_workers: usize,
    handles: Vec<thread::JoinHandle<()>>,
}

impl<I, O> ThreadPool<I, O>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    /// Spawns `num_workers` threads, each running `func` on every work item.
    pub fn new<F>(num_workers: usize, func: F) -> ThreadPool<I, O>
    where
        F: Fn(&I) -> O + Send + Sync + 'static,
    {
        let (work_tx, work_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        let work_rx = Arc::new(Mutex::new(work_rx));
        let res_rx = Arc::new(Mutex::new(res_rx));
        let func = Arc::new(func);
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            let work_rx = Arc::clone(&work_rx);
            let res_tx = res_tx.clone();
            let func = Arc::clone(&func);
            let handle = thread::spawn(move ||
                loop {
                    let work = {
                        let guard = work_rx.lock().unwrap();
                        guard.recv()
                    };
                    match work {
                        Ok(work) => {
                            let res = func(&work);
                            res_tx.send((work, res)).unwrap();
                        }
                        Err(_) => break,
                    }
                }
            );
            handles.push(handle);
        }
        ThreadPool {
            work_rx,
            work_tx,
            res_rx,
            res_tx,
            num_workers,
            handles
        }
    }

    /// Sends one work item to the pool. Returns `Err` only if the pool has
    /// already been shut down (i.e. `collect` was called).
    pub fn submit(&self, work: I) -> Result<(), &str> {
        match self.work_tx.send(work) {
            Ok(_) => Ok(()),
            Err(_) => Err("Work queue is full"),
        }
    }

    /// Signals shutdown, waits for all workers to finish, and returns every
    /// `(input, output)` pair. Consumes `self`.
    pub fn collect(self) -> Vec<(I, O)> {
        drop(self.work_tx);
        drop(self.res_tx);
        let mut res = Vec::new();
        loop {
            let work = {
                let guard = self.res_rx.lock().unwrap().recv();
                guard
            };
            match work {
                Ok(work) => res.push(work),
                Err(_) => break,
            }
        }
        for handle in self.handles {
            handle.join().unwrap();
        }
        res
    }
}
