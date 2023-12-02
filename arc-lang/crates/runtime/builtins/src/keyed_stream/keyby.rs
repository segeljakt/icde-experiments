use runner::context::Context;

use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn keyby<K1: Data>(
        mut self,
        ctx: &mut Context,
        fun: fn(T) -> K1,
    ) -> KeyedStream<K1, T> {
        let (tx1, rx1) = crate::keyed_stream::new();
        ctx.join_set.spawn(async move {
            loop {
                match self.0.recv().await.unwrap() {
                    KeyedEvent::Data(t, _, v) => {
                        let k = fun(v.clone());
                        tx1.send(KeyedEvent::Data(t, k, v)).await;
                    }
                    KeyedEvent::Watermark(t) => {
                        tx1.send(KeyedEvent::Watermark(t)).await;
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx1.send(KeyedEvent::Snapshot(i)).await;
                    }
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
