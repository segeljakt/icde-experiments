use runner::context::Context;

use crate::traits::Data;
use crate::traits::Key;

use super::KeyedCollector;
use super::KeyedEvent;
use super::KeyedStream;

impl<K: Key, T: Data> KeyedStream<K, T> {
    pub fn simulator<F, Fut>(ctx: &mut Context, f: F) -> KeyedStream<K, T>
    where
        F: Fn(KeyedCollector<K, T>) -> Fut,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = super::new();
        ctx.join_set.spawn(f(tx));
        rx
    }

    pub fn drain(mut self, ctx: &mut Context) {
        ctx.join_set.spawn(async move {
            loop {
                if let KeyedEvent::Sentinel = self.recv().await {
                    break;
                }
            }
        });
    }
}
