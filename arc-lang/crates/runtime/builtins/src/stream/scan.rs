use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn scan<A: Data>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        init: A,
        fun: fn(T, A) -> A,
    ) -> Stream<A> {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        join_set.spawn(async move {
            let mut acc = init;
            loop {
                match self.0.recv().await.unwrap() {
                    Event::Data(t, v) => {
                        acc = fun(v.clone(), acc);
                        tx1.send(Event::Data(t, acc.clone())).await.unwrap();
                    }
                    Event::Watermark(t) => {
                        tx1.send(Event::Watermark(t)).await.unwrap();
                    }
                    Event::Snapshot(i) => {
                        tx1.send(Event::Snapshot(i)).await.unwrap();
                    }
                    Event::Sentinel => {
                        tx1.send(Event::Sentinel).await.unwrap();
                        break;
                    }
                }
            }
        });
        Stream(rx1)
    }
}
