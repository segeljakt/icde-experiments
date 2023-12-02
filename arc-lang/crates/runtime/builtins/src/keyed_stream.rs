mod filter;
mod fork;
mod join;
mod map;
mod merge;
mod scan;
mod unkey;
mod window;
mod keyby;
mod sink;
mod flat_map;
mod simulator;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::time::Time;
use crate::traits::Data;
use crate::traits::Key;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum KeyedEvent<K, T> {
    Data(Time, K, T),
    Watermark(Time),
    Snapshot(usize),
    Sentinel,
}

pub struct KeyedStream<K: Data, T: Data>(pub(crate) Receiver<KeyedEvent<K, T>>);

pub struct KeyedCollector<K: Data, T: Data>(pub(crate) Sender<KeyedEvent<K, T>>);

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub async fn recv(&mut self) -> KeyedEvent<K, T> {
        self.0.recv().await.unwrap_or(KeyedEvent::Sentinel)
    }
}

impl<K: Data, T: Data> KeyedCollector<K, T> {
    pub async fn send(&self, event: KeyedEvent<K, T>) {
        self.0.send(event).await.ok();
    }
}

pub(crate) fn new<K: Data, T: Data>() -> (KeyedCollector<K, T>, KeyedStream<K, T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (KeyedCollector(tx), KeyedStream(rx))
}
