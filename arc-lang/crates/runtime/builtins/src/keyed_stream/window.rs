use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::collections::HashMap;

use runner::context::Context;

use crate::aggregator::Aggregator;
use crate::assigner::Assigner;
use crate::duration::Duration;
use crate::time::Time;
use crate::traits::Data;
use crate::traits::Key;
use crate::vec::Vec;

use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn window<P, O>(
        self,
        ctx: &mut Context,
        discretizer: Assigner,
        aggregator: Aggregator<fn(T) -> P, fn(P, P) -> P, fn(P) -> O, fn(Vec<T>) -> O>,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        match aggregator {
            Aggregator::Incremental {
                lift,
                combine,
                lower,
            } => match discretizer {
                Assigner::Tumbling { length } => {
                    self.incremental_tumbling_window(ctx, length, lift, combine, lower)
                }
                Assigner::Sliding { .. } => todo!(),
                Assigner::Session { .. } => todo!(),
                Assigner::Counting { .. } => todo!(),
                Assigner::Moving { .. } => todo!(),
            },
            Aggregator::Holistic { compute } => match discretizer {
                Assigner::Tumbling { length } => {
                    self.holistic_tumbling_window(ctx, length, compute)
                }
                Assigner::Sliding { .. } => todo!(),
                Assigner::Session { .. } => todo!(),
                Assigner::Counting { .. } => todo!(),
                Assigner::Moving { .. } => todo!(),
            },
        }
    }

    pub fn incremental_tumbling_window<P, O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        lift: fn(T) -> P,
        combine: fn(P, P) -> P,
        lower: fn(P) -> O,
    ) -> KeyedStream<K, O>
    where
        P: Data,
        O: Data,
    {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        ctx.join_set.spawn(async move {
            let mut aggs: BTreeMap<Time, HashMap<K, P>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let t0 = time.div_floor(duration) * duration;
                        match aggs.entry(t0).or_insert_with(HashMap::new).entry(key) {
                            Entry::Occupied(mut entry) => {
                                *entry.get_mut() = combine(entry.get().clone(), lift(data));
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(lift(data));
                            }
                        }
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some(entry) = aggs.first_entry() {
                            let t1 = *entry.key() + duration;
                            if t1 < time {
                                for (key, p) in entry.remove() {
                                    let data = lower(p);
                                    tx1.send(KeyedEvent::Data(t1, key, data)).await.unwrap();
                                }
                                tx1.send(KeyedEvent::Watermark(time)).await.unwrap();
                            } else {
                                tx1.send(KeyedEvent::Watermark(time)).await.unwrap();
                                break;
                            }
                        }
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

    pub fn holistic_tumbling_window<O>(
        mut self,
        ctx: &mut Context,
        duration: Duration,
        compute: fn(Vec<T>) -> O,
    ) -> KeyedStream<K, O>
    where
        O: Data,
    {
        let (tx1, rx1) = tokio::sync::mpsc::channel(100);
        ctx.join_set.spawn(async move {
            let mut aggs: BTreeMap<Time, HashMap<K, std::vec::Vec<(Time, T)>>> = BTreeMap::new();
            loop {
                match self.recv().await {
                    KeyedEvent::Data(time, key, data) => {
                        let t0 = time.div_floor(duration) * duration;
                        aggs.entry(t0)
                            .or_insert_with(HashMap::new)
                            .entry(key)
                            .or_insert_with(std::vec::Vec::new)
                            .push((time, data));
                    }
                    KeyedEvent::Watermark(time) => {
                        while let Some(entry) = aggs.first_entry() {
                            let t1 = *entry.key() + duration;
                            if t1 < time {
                                for (key, mut vs) in entry.remove() {
                                    vs.sort_by_key(|(t, _)| *t);
                                    let vs = Vec::from(
                                        vs.into_iter()
                                            .map(|(_, v)| v)
                                            .collect::<std::vec::Vec<_>>(),
                                    );
                                    let data = compute(vs);
                                    tx1.send(KeyedEvent::Data(t1, key, data)).await.unwrap();
                                }
                            } else {
                                break;
                            }
                        }
                        tx1.send(KeyedEvent::Watermark(time)).await.unwrap();
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
