use crate::{engine::Command, shard_engine::shard::Shard};

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
        if let Err(e) = self.shards[shard_id].cmd_tx.send(cmd).await {
            eprintln!("Shard {} channel send failed: {}", shard_id, e);
        }
    }

    fn compute_shard_id(&self, key: &str) -> usize {
        let hash = fxhash::hash64(key.as_bytes());
        (hash as usize) % self.shards.len()
    }
}
