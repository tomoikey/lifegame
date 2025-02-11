use crate::cell::OwnedCells;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct Holder<const MAX: usize> {
    millis_per_frame: u64,
    queue: Arc<Mutex<VecDeque<OwnedCells>>>,
    /// From Calculator
    receiver: Arc<Mutex<Receiver<OwnedCells>>>,
    /// To Drawer
    sender: Arc<Mutex<Sender<OwnedCells>>>,
}

impl<const MAX: usize> Holder<MAX> {
    pub fn new(
        millis_per_frame: u64,
        receiver: Receiver<OwnedCells>,
        sender: Sender<OwnedCells>,
    ) -> Self {
        Self {
            millis_per_frame,
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(MAX))),
            receiver: Arc::new(Mutex::new(receiver)),
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    pub async fn run(self) {
        let queue = self.queue.clone();
        let receiver = self.receiver.clone();
        let sender = self.sender.clone();

        let receiver_thread = {
            let queue = queue.clone();
            tokio::spawn(async move {
                loop {
                    if queue.lock().await.len() == MAX {
                        continue;
                    }
                    let cells = receiver.lock().await.recv().await.expect("channel closed");
                    queue.lock().await.push_front(cells);
                }
            })
        };

        let sender_thread = {
            let queue = queue.clone();
            let sender = sender.clone();
            tokio::spawn(async move {
                loop {
                    let queue = queue.lock().await.pop_back();
                    if let Some(cells) = queue {
                        sender
                            .lock()
                            .await
                            .send(cells)
                            .await
                            .expect("channel closed");
                    }

                    sleep(Duration::from_millis(self.millis_per_frame)).await;
                }
            })
        };

        select! {
            _ = receiver_thread => {
                panic!("receiver_thread panicked");
            },
            _ = sender_thread => {
                panic!("sender_thread panicked");
            },
        }
    }
}
