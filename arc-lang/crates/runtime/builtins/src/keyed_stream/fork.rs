use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn split(mut self, join_set: &mut tokio::task::JoinSet<()>) -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        let (tx2, rx2) = tokio::sync::mpsc::channel(100);
        join_set.spawn(async move {
            loop {
                let (l, r) = match self.recv().await {
                    KeyedEvent::Data(t, k1, v1) => {
                        let k2 = k1.clone();
                        let v2 = v1.clone();
                        tokio::join!(
                            tx1.send(KeyedEvent::Data(t, k2, v2)),
                            tx2.send(KeyedEvent::Data(t, k1, v1)),
                        )
                    }
                    KeyedEvent::Watermark(t) => {
                        tokio::join!(
                            tx1.send(KeyedEvent::Watermark(t)),
                            tx2.send(KeyedEvent::Watermark(t))
                        )
                    }
                    KeyedEvent::Snapshot(i) => {
                        tokio::join!(
                            tx1.send(KeyedEvent::Snapshot(i)),
                            tx2.send(KeyedEvent::Snapshot(i))
                        )
                    }
                    KeyedEvent::Sentinel => {
                        tokio::join!(
                            tx1.send(KeyedEvent::Sentinel),
                            tx2.send(KeyedEvent::Sentinel)
                        )
                    }
                };
                l.unwrap();
                r.unwrap();
            }
        });
        (Self(rx1), Self(rx2))
    }
}
