use crate::keyed_stream::KeyedEvent;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn merge(mut self, join_set: &mut tokio::task::JoinSet<()>, mut other: Self) -> Self {
        let (tx2, rx2) = tokio::sync::mpsc::channel(100);
        join_set.spawn_local(async move {
            loop {
                let event = tokio::select! {
                    event = self.recv() => {
                        if let KeyedEvent::Sentinel = event {
                            other.recv().await
                        } else {
                            event
                        }
                    },
                    event = other.recv() => {
                        if let KeyedEvent::Sentinel = event {
                            self.recv().await
                        } else {
                            event
                        }
                    },
                };
                match event {
                    KeyedEvent::Data(t, k1, v1) => {
                        tx2.send(KeyedEvent::Data(t, k1, v1)).await.unwrap();
                    }
                    KeyedEvent::Watermark(t) => {
                        tx2.send(KeyedEvent::Watermark(t)).await.unwrap();
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx2.send(KeyedEvent::Snapshot(i)).await.unwrap();
                    }
                    KeyedEvent::Sentinel => {
                        tx2.send(KeyedEvent::Sentinel).await.unwrap();
                        break;
                    }
                }
            }
        });
        Self(rx2)
    }
}
