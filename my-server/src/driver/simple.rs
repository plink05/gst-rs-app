use my_common::thread::IntoThread;

pub struct SimpleDriver {

}

impl SimpleDriver {
    pub fn new() -> Self {
        Self { }
    }
}

impl IntoThread for SimpleDriver {
    fn thread_name(&self) -> String {
        "simple".to_string()
    }

    fn main(self, continue_progress: std::sync::Arc<std::sync::atomic::AtomicBool>) -> anyhow::Result<()> {
        while continue_progress.load(std::sync::atomic::Ordering::Relaxed) {
            println!("simple");
        }

        Ok(())
    }
}
