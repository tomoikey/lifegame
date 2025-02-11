use crate::cell::OwnedCells;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

pub struct Queue<const MAX_QUEUE_SIZE: usize> {
    queue: Arc<Mutex<VecDeque<OwnedCells>>>,
    cell_receiver: Receiver<OwnedCells>,
    cell_sender: Sender<OwnedCells>,
    schedule_receiver: Receiver<()>,
}

impl<const MAX_QUEUE_SIZE: usize> Queue<MAX_QUEUE_SIZE> {
    pub fn new(
        cell_receiver: Receiver<OwnedCells>,
        cell_sender: Sender<OwnedCells>,
        schedule_receiver: Receiver<()>,
    ) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_SIZE))),
            cell_receiver,
            cell_sender,
            schedule_receiver,
        }
    }

    pub async fn run(mut self) {
        let receiver_thread = {
            let queue = Arc::clone(&self.queue);
            tokio::spawn(async move {
                loop {
                    if queue.lock().await.len() == MAX_QUEUE_SIZE {
                        continue;
                    }
                    let cells = self.cell_receiver.recv().await.expect("channel closed");
                    queue.lock().await.push_front(cells);
                }
            })
        };

        let sender_thread = {
            let queue = self.queue;
            tokio::spawn(async move {
                loop {
                    let _ = self.schedule_receiver.recv().await.expect("channel closed");
                    let queue = queue.lock().await.pop_back();
                    if let Some(cells) = queue {
                        self.cell_sender.send(cells).await.expect("channel closed");
                    }
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
