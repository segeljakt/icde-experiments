use runner::context::Context;

use crate::option::Option;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn filter_map<O>(
        mut self,
        ctx: &mut Context,
        f: impl Fn(T) -> Option<O> + Send + 'static,
    ) -> Stream<O>
    where
        O: Data,
    {
        let (tx1, rx1) = super::new();
        ctx.join_set.spawn(async move {
            loop {
                match self.recv().await {
                    Event::Data(t, v) => {
                        if let Some(v) = f(v.clone()).0 {
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
