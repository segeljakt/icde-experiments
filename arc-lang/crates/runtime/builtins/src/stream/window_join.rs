use std::collections::BTreeMap;
use std::collections::HashMap;

use runner::context::Context;

use crate::duration::Duration;
use crate::stream::Event;
use crate::time::Time;
use crate::traits::Data;
use crate::traits::Key;

use fxhash::FxBuildHasher;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn window_join<R, K, O>(
        mut self,
        ctx: &mut Context,
        mut other: Stream<R>,
        left_key: impl Fn(&T) -> K + Send + 'static,
        right_key: impl Fn(&R) -> K + Send + 'static,
        duration: Duration,
        joiner: impl Fn(T, R) -> O + Send + 'static,
    ) -> Stream<O>
    where
        R: Data,
        K: Data + Key,
        O: Data,
    {
        let (tx, rx) = super::new();
        ctx.join_set.spawn(async move {
            let mut s: BTreeMap<Time, HashMap<K, (Vec<T>, Vec<R>), FxBuildHasher>> = BTreeMap::new();
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            let mut done_l = false;
            let mut done_r = false;
            loop {
                tokio::select! {
                    event = self.recv(), if !done_l => match event {
                        Event::Data(time, data) => {
                            let key = left_key(&data);
                            let t0 = time.div_floor(duration) * duration;
                            let (lvec, rvec) = get_state(&mut s, t0, key);
                            for r in rvec.iter() {
                                tx.send(Event::Data(t0, joiner(data.deep_clone(), r.deep_clone()))).await;
                            }
                            lvec.push(data);
                        }
                        Event::Watermark(t) => {
                            if t < r_watermark {
                                gc_state(&mut s, t, duration);
                                tx.send(Event::Watermark(t)).await;
                            } else if l_watermark < r_watermark && r_watermark < t {
                                tx.send(Event::Watermark(r_watermark)).await
                            }
                            l_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_r {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_l = true;
                        }
                        Event::Snapshot(_) => {
                            unimplemented!()
                        }
                    },
                    event = other.recv(), if !done_r => match event {
                        Event::Data(time, data) => {
                            let key = right_key(&data);
                            let t0 = time.div_floor(duration) * duration;
                            let (lvec, rvec) = get_state(&mut s, t0, key);
                            for l in lvec.iter() {
                                tx.send(Event::Data(t0, joiner(l.deep_clone(), data.deep_clone()))).await;
                            }
                            rvec.push(data);
                        }
                        Event::Watermark(t) => {
                            if t < l_watermark {
                                gc_state(&mut s, t, duration);
                                tx.send(Event::Watermark(t)).await;
                            } else if r_watermark < l_watermark && l_watermark < t {
                                tx.send(Event::Watermark(l_watermark)).await
                            }
                            r_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_l {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_r = true;
                        }
                        Event::Snapshot(_) => {
                            unimplemented!()
                        }
                    },
                };
            }
        });
        rx
    }
}

enum State<L, R> {
    Left(L),
    Right(R),
    Empty,
}

impl<L, R> Default for State<L, R> {
    fn default() -> Self {
        State::Empty
    }
}

impl<T: Data> Stream<T> {
    pub fn window_join_distinct<R, K, O>(
        mut self,
        ctx: &mut Context,
        mut other: Stream<R>,
        left_key: impl Fn(&T) -> K + Send + 'static,
        right_key: impl Fn(&R) -> K + Send + 'static,
        duration: Duration,
        joiner: impl Fn(T, R) -> O + Send + 'static,
    ) -> Stream<O>
    where
        R: Data,
        K: Data + Key,
        O: Data,
    {
        let (tx, rx) = super::new();
        ctx.join_set.spawn(async move {
            let mut s: BTreeMap<Time, HashMap<K, State<T, R>, FxBuildHasher>> = BTreeMap::new();
            let mut l_watermark = Time::zero();
            let mut r_watermark = Time::zero();
            let mut done_l = false;
            let mut done_r = false;
            loop {
                tokio::select! {
                    event = self.recv(), if !done_l => match event {
                        Event::Data(time, data) => {
                            let key = left_key(&data);
                            let t0 = time.div_floor(duration) * duration;
                            let skey = get_state(&mut s, t0, key);
                            match skey {
                                State::Left(_) => {
                                    unreachable!()
                                }
                                State::Right(_) => {
                                    let State::Right(r) = std::mem::take(skey) else { unreachable!() };
                                    tx.send(Event::Data(t0, joiner(data, r))).await;
                                }
                                State::Empty => {
                                    *skey = State::Left(data);
                                }
                            }
                        }
                        Event::Watermark(t) => {
                            if t < r_watermark {
                                gc_state(&mut s, t, duration);
                                tx.send(Event::Watermark(t)).await;
                            } else if l_watermark < r_watermark && r_watermark < t {
                                tx.send(Event::Watermark(r_watermark)).await
                            }
                            l_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_r {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_l = true;
                        }
                        Event::Snapshot(_) => {
                            unimplemented!()
                        }
                    },
                    event = other.recv(), if !done_r => match event {
                        Event::Data(time, data) => {
                            let key = right_key(&data);
                            let t0 = time.div_floor(duration) * duration;
                            let skey = get_state(&mut s, t0, key);
                            match skey {
                                State::Left(_) => {
                                    let State::Left(l) = std::mem::take(skey) else { unreachable!() };
                                    tx.send(Event::Data(t0, joiner(l, data))).await;
                                }
                                State::Right(_) => {
                                    unreachable!()
                                }
                                State::Empty => {
                                    *skey = State::Right(data);
                                }
                            }
                        }
                        Event::Watermark(t) => {
                            if t < l_watermark {
                                gc_state(&mut s, t, duration);
                                tx.send(Event::Watermark(t)).await;
                            } else if r_watermark < l_watermark && l_watermark < t {
                                tx.send(Event::Watermark(l_watermark)).await
                            }
                            r_watermark = t;
                        }
                        Event::Sentinel => {
                            if done_l {
                                tx.send(Event::Sentinel).await;
                                break;
                            }
                            done_r = true;
                        }
                        Event::Snapshot(_) => {
                            unimplemented!()
                        }
                    },
                };
            }
        });
        rx
    }
}

fn gc_state<T>(s: &mut BTreeMap<Time, T>, time: Time, duration: Duration) {
    while let Some(entry) = s.first_entry() {
        let t1 = *entry.key() + duration;
        if t1 < time {
            entry.remove();
        } else {
            break;
        }
    }
}

fn get_state<K: Key, T: Default>(
    s: &mut BTreeMap<Time, HashMap<K, T, FxBuildHasher>>,
    time: Time,
    key: K,
) -> &mut T {
    s.entry(time)
        .or_insert_with(HashMap::default)
        .entry(key)
        .or_insert_with(T::default)
}
