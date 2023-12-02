pub struct Context {
    pub join_set: tokio::task::JoinSet<()>,
    pub rx: tokio::sync::broadcast::Receiver<()>,
}

impl Context {
    pub fn new(rx: tokio::sync::broadcast::Receiver<()>) -> Self {
        Self {
            join_set: tokio::task::JoinSet::new(),
            rx,
        }
    }
}
