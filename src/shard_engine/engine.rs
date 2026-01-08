use tokio::sync::mpsc;

use crate::{engine, shard_engine::shard::Shard};

pub fn spawn_shards(n: usize) -> Vec<Shard> {
    let mut shards = Vec::with_capacity(n);
    for id in 0..n {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (wal_tx, wal_rx) = mpsc::channel(100);
        engine::start_wal_task(id, wal_rx);
        engine::start_engine(id, cmd_rx, wal_tx);
        shards.push(Shard::new(id, cmd_tx));
    }
    shards
}
