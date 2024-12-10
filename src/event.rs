use std::time::Duration;

use anyhow::{anyhow, Result};
use crossterm::event::Event as CrosstermEvent;
use futures::{FutureExt, StreamExt};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time,
};

pub enum Event {
    Tick,
    Terminal(CrosstermEvent),
}

#[allow(dead_code)]
pub struct EventHandler {
    sender: UnboundedSender<Event>,
    receiver: UnboundedReceiver<Event>,
    handler: JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tps: u64) -> Self {
        let tick_duration = Duration::from_micros(1_000_000 / tps);

        // Create message channel
        let (tx, rx) = mpsc::unbounded_channel();
        let tx2 = tx.clone();

        // Create async task
        let handler = tokio::spawn(async move {
            let mut tick = time::interval(tick_duration);
            let mut reader = crossterm::event::EventStream::new();

            loop {
                tokio::select! {
                    _ = tx2.closed() => {
                        break;
                    }
                    _ = tick.tick() => {
                        tx2.send(Event::Tick).unwrap();
                    }
                    Some(Ok(ev)) = reader.next().fuse() => {
                        tx2.send(Event::Terminal(ev)).unwrap();
                    }
                }
            }
        });

        Self {
            sender: tx,
            receiver: rx,
            handler,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow!("fail to receive next event"))
    }
}
