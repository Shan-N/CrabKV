use tokio::sync::mpsc::Sender;

use crate::engine::Command;

pub struct Shard {
    id: usize,
    pub cmd_tx: Sender<Command>,
}

impl Shard {
    pub fn new(id: usize, cmd_tx: Sender<Command>) -> Self {
        Self { id, cmd_tx }
    }
}
