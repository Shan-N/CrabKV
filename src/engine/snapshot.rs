use std::collections::HashMap;
use std::fs::{self, rename};

pub fn save_snapshot(shard_id: usize, db: &HashMap<String, String>, ttl_db: &HashMap<String, u64>) {
    let contents = serde_json::to_string(&(db, ttl_db)).unwrap();
    fs::write(format!("snapshot_{}.json.tmp", shard_id), contents).unwrap();
    rename(
        format!("snapshot_{}.json.tmp", shard_id),
        format!("snapshot_{}.json", shard_id),
    )
    .unwrap();
}
