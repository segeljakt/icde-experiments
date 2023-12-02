use runner::context::Context;

use crate::keyed_stream::KeyedEvent;
use crate::keyed_stream::KeyedStream;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn keyby<K: Data>(mut self, ctx: &mut Context, fun: fn(T) -> K) -> KeyedStream<K, T> {
        let (tx1, rx1) = crate::keyed_stream::new();
        ctx.join_set.spawn(async move {
            loop {
                match self.0.recv().await.unwrap() {
                    Event::Data(t, v) => {
                        let k = fun(v.clone());
                        tx1.send(KeyedEvent::Data(t, k, v)).await;
                    }
                    Event::Watermark(t) => {
                        tx1.send(KeyedEvent::Watermark(t)).await;
                    }
                    Event::Snapshot(i) => {
                        tx1.send(KeyedEvent::Snapshot(i)).await;
                    }
                    Event::Sentinel => {
                        tx1.send(KeyedEvent::Sentinel).await;
                        break;
                    }
                }
            }
        });
        rx1
    }
}
