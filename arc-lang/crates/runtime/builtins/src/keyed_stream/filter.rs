use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn filter(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        f: fn(T) -> bool,
    ) -> KeyedStream<K, T> {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        join_set.spawn_local(async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        if f(v.clone()) {
                            tx1.send(KeyedEvent::Data(t, k, v)).await.unwrap();
                        }
                    }
                    KeyedEvent::Watermark(t) => tx1.send(KeyedEvent::Watermark(t)).await.unwrap(),
                    KeyedEvent::Snapshot(i) => tx1.send(KeyedEvent::Snapshot(i)).await.unwrap(),
                    KeyedEvent::Sentinel => {
                        tx1.send(KeyedEvent::Sentinel).await.unwrap();
                        break;
                    }
                }
            }
        });
        KeyedStream(rx1)
    }
}
