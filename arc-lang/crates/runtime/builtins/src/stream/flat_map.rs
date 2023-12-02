use crate::traits::Data;
use crate::vec::Vec;

use super::Event;
use super::Stream;

impl<T> Stream<T>
where
    T: Data,
{
    pub fn flat_map<O>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        f: fn(T) -> Vec<O>,
    ) -> Stream<O>
    where
        O: Data,
    {
        let (tx1, rx1) = super::new();
        join_set.spawn(async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        for v in f(v).iter() {
                            tx1.send(Event::Data(t, v)).await;
                        }
                    }
                    Event::Watermark(t) => tx1.send(Event::Watermark(t)).await,
                    Event::Snapshot(i) => tx1.send(Event::Snapshot(i)).await,
                    Event::Sentinel => {
                        tx1.send(Event::Sentinel).await;
                        break;
                    }
                }
            }
        });
        rx1
    }
}
