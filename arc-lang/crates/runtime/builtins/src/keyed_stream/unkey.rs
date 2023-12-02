use runner::context::Context;

use crate::stream::Event;
use crate::stream::Stream;
use crate::traits::Data;
use crate::traits::Key;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn unkey(mut self, ctx: &mut Context) -> Stream<T> {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        ctx.join_set.spawn(async move {
            loop {
                match self.recv().await {
                    KeyedEvent::Data(t, _, v) => {
                        tx1.send(Event::Data(t, v)).await.unwrap();
                    }
                    KeyedEvent::Watermark(t) => {
                        tx1.send(Event::Watermark(t)).await.unwrap();
                    }
                    KeyedEvent::Snapshot(i) => {
                        tx1.send(Event::Snapshot(i)).await.unwrap();
                    }
                    KeyedEvent::Sentinel => {
                        tx1.send(Event::Sentinel).await.unwrap();
                        break;
                    }
                }
            }
        });
        Stream(rx1)
    }
}
