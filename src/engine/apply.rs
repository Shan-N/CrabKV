use super::ParsedCommand;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn apply_db(
    db: &mut HashMap<String, String>,
    ttl_db: &mut HashMap<String, u64>,
    expiry_heap: &mut BinaryHeap<Reverse<(u64, String)>>,
    cmd: ParsedCommand,
) {
    match cmd {
        ParsedCommand::Set { key, value } => {
            db.insert(key.clone(), value);
            ttl_db.remove(&key);
        }
        ParsedCommand::SetEx { key, value, ttl } => {
            let expiry = now_ms() + ttl * 1000;
            db.insert(key.clone(), value);
            ttl_db.insert(key.clone(), expiry);
            expiry_heap.push(Reverse((expiry, key.clone())));
        }
        ParsedCommand::Expire { key, ttl } => {
            if db.contains_key(&key) {
                let expiry = now_ms() + ttl * 1000;
                ttl_db.insert(key.clone(), expiry);
                expiry_heap.push(Reverse((expiry, key.clone())));
            }
        }
        ParsedCommand::Del { key } => {
            db.remove(&key);
            ttl_db.remove(&key);
        }
        _ => {}
    }
}
