use my_common::thread::IntoThread;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use anyhow::Result;
extern crate gstreamer as gst;
use gst::prelude::*;

pub struct GstDriver {

}

impl GstDriver {
    pub fn new() -> Self {
        Self { }
    }
}

impl IntoThread for GstDriver {
    fn thread_name(&self) -> String {
        "gst-pipeline".to_string()
    }

    fn main(self, continue_progress: Arc<AtomicBool>) -> anyhow::Result<()> {
        gst::init().unwrap();

        // Build the pipeline
        let pipeline = gst::parse::launch("playbin uri=file:///Users/patricklink/Downloads/file_example_MP4_480_1_5MG.mp4").unwrap();


        // Start playing
        pipeline.set_state(gst::State::Playing).unwrap();

        // Wait until error or EOS
        let bus = pipeline.bus().unwrap();

        while continue_progress.load(std::sync::atomic::Ordering::Relaxed) {
            for msg in bus.iter_timed(gst::ClockTime::ZERO) {
                match msg.view() {
                    gst::MessageView::Eos(..) => break,
                    gst::MessageView::Error(err) => {
                        println!(
                            "Error received from element {:?}: {}",
                            err.src().map(|s| s.path_string()),
                            err.error()
                        );
                        println!("Debugging information: {:?}", err.debug());
                        break;
                    }
                    _ => (),
                }
            }
        }

        // Free resources
        pipeline.set_state(gst::State::Null).unwrap();
        Ok(())
    }
}
