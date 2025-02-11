use crate::cell::OwnedCells;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, MutexGuard};
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

    async fn push(mut queue: MutexGuard<'_, VecDeque<OwnedCells>>, cells: OwnedCells) {
        if queue.len() == MAX {
            queue.remove(0);
        }
        queue.push_front(cells);
    }

    async fn send_to_drawer(
        mut queue: MutexGuard<'_, VecDeque<OwnedCells>>,
        sender: MutexGuard<'_, Sender<OwnedCells>>,
    ) {
        let next = queue.pop_back();
        if let Some(cells) = next {
            drop(queue);
            sender.send(cells).await.expect("channel closed");
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
                    let queue = queue.lock().await;
                    Self::push(queue, cells).await;
                }
            })
        };

        let sender_thread = {
            let queue = queue.clone();
            let sender = sender.clone();
            tokio::spawn(async move {
                loop {
                    let queue = queue.lock().await;
                    let sender = sender.lock().await;
                    Self::send_to_drawer(queue, sender).await;
                    sleep(Duration::from_millis(100)).await;
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
