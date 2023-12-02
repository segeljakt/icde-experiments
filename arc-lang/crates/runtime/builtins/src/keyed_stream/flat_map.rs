use crate::traits::Data;
use crate::traits::Key;
use crate::vec::Vec;

use super::KeyedEvent;
use super::KeyedStream;

impl<K, T> KeyedStream<K, T>
where
    K: Key,
    T: Data,
{
    pub fn flat_map<O>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        f: fn(T) -> Vec<O>,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        let (tx1, rx1) = super::new();
        join_set.spawn(async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        for v in f(v).iter() {
                            tx1.send(KeyedEvent::Data(t, k.clone(), v)).await;
                        }
                    }
                    KeyedEvent::Watermark(t) => tx1.send(KeyedEvent::Watermark(t)).await,
                    KeyedEvent::Snapshot(i) => tx1.send(KeyedEvent::Snapshot(i)).await,
                    KeyedEvent::Sentinel => {
                        tx1.send(KeyedEvent::Sentinel).await;
                        break;
                    }
                }
            }
        });
        rx1
    }
}
