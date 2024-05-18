use anyhow::{anyhow, Result};
use crossbeam::channel::tick;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
    time::Duration,
};
use thread_priority::{ThreadBuilderExt, ThreadPriority, ThreadPriorityValue};
use tracing::{error, info};

pub trait IntoThread {
    fn thread_name(&self) -> String;

    fn thread_priority() -> ThreadPriority {
        ThreadPriority::Crossplatform(ThreadPriorityValue::try_from(40u8).unwrap())
    }

    fn main(self, continue_progress: Arc<AtomicBool>) -> Result<()>;

    fn into_thread(self, continue_progress: Arc<AtomicBool>) -> JoinHandle<Result<()>>
    where
        Self: Sized + Send + 'static,
    {
        let priority = Self::thread_priority();
        let name = self.thread_name();
        std::thread::Builder::new()
            .name(name)
            .spawn_with_priority(priority, move |_| self.main(continue_progress))
            .unwrap()
    }
}

pub struct ThreadManager {
    threads: Vec<JoinHandle<anyhow::Result<()>>>,
    continue_progress: Arc<AtomicBool>,

}

impl ThreadManager {
    pub fn new(continue_progress: Arc<AtomicBool>) -> Self {
        Self {
            threads: Vec::new(),
            continue_progress,
        }
    }

    pub fn spawn_into_thread<T>(&mut self, into_thread: T)
    where 
        T: IntoThread + std::marker::Send + 'static,
    {
        let continue_progress = Arc::clone(&self.continue_progress);
        self.threads
            .push(into_thread.into_thread(continue_progress));
    }
    
    pub fn add_handle(&mut self, handle: JoinHandle<Result<()>>) {
        self.threads.push(handle);
    }

    pub fn join_all(self) -> Result<()> {
        let mut active_threads = self.threads;
        let mut exit_result = Ok(());

        let ticker = tick(Duration::from_millis(100));
        loop {
            let _ = ticker.recv();

            if active_threads.is_empty() {
                break exit_result;
            }
            active_threads = active_threads
                .into_iter()
                .filter_map(|handle| {
                    if handle.is_finished() {
                        let name = handle.thread().name().unwrap().to_owned();

                        match handle.join() {
                            Ok(Ok(_)) => {
                                info!("Thread {name:?} ended successfully -- other threads will continue");
                                return None;
                            }
                            Ok(Err(e)) => {
                                error!("Thread {name:?} ended unsuccessfuly -- other threads will be commanded to stop: {e:?}");
                                if exit_result.is_ok() {
                                    exit_result = Err(e);
                                }
                            }
                            Err(_) => {
                                error!("Thread {name:?} failed to join");
                                if exit_result.is_ok() {
                                    exit_result = Err(anyhow!("Thread {name:?} failed to join"));
                                }
                            }
                        }
                        if self
                            .continue_progress
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            self.continue_progress
                                .store(false, std::sync::atomic::Ordering::Release);
                        }

                        None
                    } else {
                        Some(handle)
                    }

                })
                .collect();
        }
    }


}


