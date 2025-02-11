use crate::cell::OwnedCells;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct Holder<const MAX: usize> {
    queue: Arc<Mutex<VecDeque<OwnedCells>>>,
    /// From Calculator
    receiver: Arc<Mutex<Receiver<OwnedCells>>>,
    /// To Drawer
    sender: Arc<Mutex<Sender<OwnedCells>>>,
}

impl<const MAX: usize> Holder<MAX> {
    pub fn new(receiver: Receiver<OwnedCells>, sender: Sender<OwnedCells>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(MAX))),
            receiver: Arc::new(Mutex::new(receiver)),
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    async fn push(&self, cells: OwnedCells) {
        let mut queue = self.queue.lock().await;
        if queue.len() == MAX {
            queue.remove(0);
        }
        queue.push_front(cells);
    }

    async fn send_to_drawer(&self) {
        let next = self.queue.lock().await.pop_back();
        if let Some(cells) = next {
            self.sender
                .lock()
                .await
                .send(cells)
                .await
                .expect("channel closed");
        }
    }

    pub async fn run(self) {
        let this = Arc::new(Mutex::new(self));

        let receiver_thread = {
            let this = Arc::clone(&this);
            tokio::spawn(async move {
                let this = this.lock().await;
                let cells = this
                    .receiver
                    .lock()
                    .await
                    .recv()
                    .await
                    .expect("channel closed");
                this.push(cells).await;
            })
        };

        let sender_thread = {
            let this = Arc::clone(&this);
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_millis(100)).await;
                    this.lock().await.send_to_drawer().await;
                }
            })
        };

        select! {
            _ = receiver_thread => {},
            _ = sender_thread => {},
        }
    }
}
