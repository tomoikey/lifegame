use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

pub struct Scheduler {
    millis_per_frame: u64,
    sender: Sender<()>,
}

impl Scheduler {
    pub fn new(millis_per_frame: u64, sender: Sender<()>) -> Self {
        Self {
            millis_per_frame,
            sender,
        }
    }

    pub async fn run(self) {
        loop {
            self.sender.send(()).await.expect("channel closed");
            sleep(Duration::from_millis(self.millis_per_frame)).await;
        }
    }
}
