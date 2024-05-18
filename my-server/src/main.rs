use my_common::thread::{ IntoThread, ThreadManager };
use std::{
    sync::Arc,
    sync::atomic::AtomicBool, thread::Thread,
};
use anyhow::Result;
use my_server::driver::simple::SimpleDriver;

fn main() -> Result<()> {
    
    let continue_progress = Arc::new(AtomicBool::new(true));

    
    let mut thread_manager = ThreadManager::new(Arc::clone(&continue_progress));

    {
        let continue_progress = Arc::clone(&continue_progress);
        ctrlc::set_handler(move || {
            continue_progress.store(false, std::sync::atomic::Ordering::Relaxed);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let simple = SimpleDriver::new();

    thread_manager.spawn_into_thread(simple);
    thread_manager.join_all()
}
