use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::time::Time;
use crate::traits::Data;
use serde::Deserialize;
use serde::Serialize;

mod filter;
mod flat_map;
mod fork;
mod keyby;
mod map;
mod merge;
mod scan;
mod sink;
mod source;
mod window;
mod window_join;
mod simulator;
mod batch;
mod filter_map;

#[derive(Debug, Serialize, Deserialize)]
pub enum Event<T> {
    Data(Time, T),
    Watermark(Time),
    Snapshot(usize),
    Sentinel,
}

pub struct Stream<T>(pub(crate) Receiver<Event<T>>);

pub struct Collector<T>(pub(crate) Sender<Event<T>>);

impl<T: Data> Stream<T> {
    pub async fn recv(&mut self) -> Event<T> {
        self.0.recv().await.unwrap_or(Event::Sentinel)
    }
}

impl<T: Data> Collector<T> {
    pub async fn send(&self, event: Event<T>) {
        self.0.send(event).await.ok();
    }
}

fn new<T: Data>() -> (Collector<T>, Stream<T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (Collector(tx), Stream(rx))
}
