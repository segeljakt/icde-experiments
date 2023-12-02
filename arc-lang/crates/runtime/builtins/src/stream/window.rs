use std::collections::BTreeMap;

use crate::duration::Duration;
use crate::time::Time;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn window<O>(
        mut self,
        join_set: &mut tokio::task::JoinSet<()>,
        duration: Duration,
        compute: fn(Vec<T>) -> O,
    ) -> Stream<O>
    where
        O: Data,
    {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        join_set.spawn(async move {
            loop {
                let mut s: BTreeMap<Time, Vec<(Time, T)>> = BTreeMap::new();
                match self.recv().await {
                    Event::Data(time, data) => {
                        let t0 = time.div_floor(duration) * duration;
                        s.entry(t0).or_insert_with(Vec::new).push((time, data));
                    }
                    Event::Watermark(time) => {
                        while let Some(entry) = s.first_entry() {
                            let t1 = *entry.key() + duration;
                            if t1 < time {
                                let mut vs = entry.remove();
                                vs.sort_by_key(|(t, _)| *t);
                                let vs = vs.into_iter().map(|(_, v)| v).collect();
                                let v = compute(vs);
                                tx1.send(Event::Data(t1, v)).await.unwrap();
                            } else {
                                break;
                            }
                        }
                        tx1.send(Event::Watermark(time)).await.unwrap();
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
