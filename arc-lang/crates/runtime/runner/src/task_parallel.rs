use crate::context::Context;

pub struct TaskParallelRunner {
    tx: std::sync::mpsc::SyncSender<()>,
}

impl TaskParallelRunner {
    pub fn new(f: impl FnOnce(&mut Context) + Send + 'static) -> Self {
        let (tx, _rx) = std::sync::mpsc::sync_channel::<()>(1);
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(async {
                let (_tx1, rx1) = tokio::sync::broadcast::channel(1);
                let mut ctx = Context::new(rx1);
                f(&mut ctx);
                // rx.recv().unwrap();
                // tx1.send(()).unwrap();
                while let Some(_) = ctx.join_set.join_next().await {}
            });
        Self { tx }
    }

    pub fn run(self) {
        self.tx.send(()).unwrap();
    }
}
