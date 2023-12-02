use crate::context::Context;

pub struct DataParallelRunner {
    txs: Vec<std::sync::mpsc::Sender<()>>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl DataParallelRunner {
    pub fn new<T: Send + 'static, const N: usize>(args: [T; N], f: fn(T, &mut Context)) -> Self {
        let mut threads = Vec::with_capacity(args.len());
        let mut txs = Vec::with_capacity(args.len());
        for arg in args {
            let (runner_tx, runner_rx) = std::sync::mpsc::channel();
            txs.push(runner_tx);
            threads.push(std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build runtime")
                    .block_on(async {
                        let (source_tx, source_rx) = tokio::sync::broadcast::channel(1);
                        let mut ctx = Context::new(source_rx);
                        let local_set = tokio::task::LocalSet::new();
                        runner_rx.recv().unwrap();
                        source_tx.send(()).unwrap();
                        local_set.run_until(async { f(arg, &mut ctx) }).await;
                        while let Some(_) = ctx.join_set.join_next().await {}
                        local_set.await;
                    });
            }));
        }
        Self { txs, threads }
    }

    pub fn run(mut self) {
        for tx in self.txs.iter() {
            tx.send(()).unwrap();
        }
        for thread in self.threads.drain(..) {
            thread.join().expect("Failed to join thread");
        }
    }
}
