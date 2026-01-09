use crate::engine::Command;
use crate::shard_engine::shard::Shard;
use tokio::sync::mpsc::error::TrySendError;

pub struct ShardRouter {
    shards: Vec<Shard>,
    shard_count: usize,
}

impl ShardRouter {
    pub fn new(shards: Vec<Shard>) -> Self {
        let shard_count = shards.len();
        Self {
            shards,
            shard_count,
        }
    }

    pub async fn route(&self, cmd: Command) {
        let key = cmd.primary_key();
        let shard_id = self.compute_shard_id(key);
        let shard = &self.shards[shard_id];

        match shard.cmd_tx.try_send(cmd) {
            Ok(_) => {}
            Err(TrySendError::Full(cmd)) => {
                // 3. BACKPRESSURE: Only await if we are truly flooded.
                if let Err(e) = shard.cmd_tx.send(cmd).await {
                    eprintln!("Shard {} channel closed: {}", shard_id, e);
                }
            }
            Err(TrySendError::Closed(_)) => {
                eprintln!("Shard {} channel closed", shard_id);
            }
        }
    }

    fn compute_shard_id(&self, key: &str) -> usize {
        let hash = fxhash::hash64(key.as_bytes());
        (hash as usize) % self.shard_count
    }
}