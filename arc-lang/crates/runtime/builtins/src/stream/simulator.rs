use runner::context::Context;

use crate::traits::Data;

use super::Collector;
use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn simulator<F, Fut>(ctx: &mut Context, f: F) -> Stream<T>
    where
        F: FnOnce(Collector<T>) -> Fut,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = super::new();
        ctx.join_set.spawn(f(tx));
        rx
    }

    pub fn drain(mut self, ctx: &mut Context) {
        ctx.join_set.spawn(async move {
            loop {
                if let Event::Sentinel = self.recv().await {
                    break;
                }
            }
        });
    }
}
