#![allow(unused)]
use std::collections::HashMap;

use crate::aggregator::Aggregator;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn scan<P, O>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        agg: Aggregator<fn(T) -> P, fn(P, P) -> P, fn(P) -> O, ()>,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        let state: HashMap<K, P> = HashMap::new();
        join_set.spawn(async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, k, v) => {
                        todo!()
                        // let p = state.entry(k.clone()).or_insert_with(identity);
                        // *p = combine(p.clone(), lift(v));
                        // tx1.send(KeyedEvent::Data(t, k, lower(p.clone())))
                        //     .await
                        //     .unwrap();
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
