use runner::context::Context;

use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn map<O>(mut self, ctx: &mut Context, f: impl Fn(T) -> O + Send + 'static) -> Stream<O>
    where
        O: Data,
    {
        let (tx1, rx1) = super::new();
        ctx.join_set.spawn(async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => tx1.send(Event::Data(t, f(v))).await,
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
