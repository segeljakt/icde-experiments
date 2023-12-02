use crate::dict::Dict;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn join<R, O>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        index: Dict<K, R>,
        merge: fn(T, R) -> O,
    ) -> KeyedStream<K, O>
    where
        R: Data,
        O: Data,
    {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        join_set.spawn(async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v0) => {
                        if let Some(v1) = index.0.get(&k) {
                            let v2 = merge(v0, v1.clone());
                            tx1.send(KeyedEvent::Data(t, k, v2)).await.unwrap();
                        }
                    }
                    KeyedEvent::Watermark(t) => {
                        tx1.send(KeyedEvent::Watermark(t)).await.unwrap();
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx1.send(KeyedEvent::Snapshot(i)).await.unwrap();
                    }
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
