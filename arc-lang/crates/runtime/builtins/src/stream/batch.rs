use runner::context::Context;
use time::OffsetDateTime;

use crate::duration::Duration;
use crate::time::Time;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn from_iter(
        ctx: &mut Context,
        iter: impl Iterator<Item = T> + Send + 'static,
        f: impl Fn(&T) -> Time + Send + 'static,
        watermark_frequency: usize,
        slack: Duration,
    ) -> Stream<T> {
        let (tx1, rx1) = super::new();
        ctx.join_set.spawn(async move {
            let mut latest_time = OffsetDateTime::UNIX_EPOCH;
            let slack = slack.to_std();
            let mut watermark = OffsetDateTime::UNIX_EPOCH;
            for (i, v) in iter.enumerate() {
                let time = f(&v);
                if time.0 < watermark {
                    continue;
                }
                if time.0 > latest_time {
                    latest_time = time.0;
                }
                if i % watermark_frequency == 0 {
                    watermark = latest_time - slack;
                    tx1.send(Event::Watermark(Time(watermark))).await;
                }
                tx1.send(Event::Data(time, v)).await;
            }
            tx1.send(Event::Sentinel).await;
        });
        rx1
    }
}
