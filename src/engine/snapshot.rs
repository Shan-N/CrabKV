use std::collections::HashMap;
use std::fs::{self, rename};

pub fn save_snapshot(db: &HashMap<String, String>, ttl_db: &HashMap<String, u64>) {
    let contents = serde_json::to_string(&(db, ttl_db)).unwrap();
    fs::write("snapshot.json.tmp", contents).unwrap();
    rename("snapshot.json.tmp", "snapshot.json").unwrap();
}
