use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{task, time};

use super::{Command, WalCommand, apply_db, parse_command, save_snapshot};
use crate::engine::apply::now_ms;

pub fn start_wal_task(shard_id: usize, mut wal_rx: Receiver<WalCommand>) {
    let filename = format!("wal_{}.log", shard_id);
    task::spawn(async move {
        let mut wal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .await
            .expect("Failed to open WAL");

        while let Some(entry) = wal_rx.recv().await {
            match entry {
                WalCommand::Write(s) => {
                    if let Err(e) = wal.write_all(s.as_bytes()).await {
                        eprintln!("WAL write error: {}", e);
                    }
                    let _ = wal.sync_all().await;
                }
                WalCommand::Truncate => {
                    if let Err(e) = wal.set_len(0).await {
                        eprintln!("WAL truncate error: {}", e);
                    }
                    let _ = wal.sync_all().await;
                    if let Err(e) = wal.seek(std::io::SeekFrom::Start(0)).await {
                        eprintln!("WAL seek error: {}", e);
                    }
                }
            }
        }
    });
}

pub fn start_engine(shard_id: usize, mut cmd_rx: Receiver<Command>, wal_tx: Sender<WalCommand>) {
    task::spawn(async move {
        let mut db: HashMap<String, String> = HashMap::new();
        let mut ttl_db: HashMap<String, u64> = HashMap::new();
        let mut expiry_heap = std::collections::BinaryHeap::new();

        let mut snapshot_interval = time::interval(Duration::from_secs(10));
        let mut cleanup_interval = time::interval(Duration::from_millis(100));

        let snapshot_file = format!("snapshot_{}.json", shard_id);

        // --- 1. Load Snapshot (Fixed to handle TTLs) ---
        if let Ok(snapshot_data) = tokio::fs::read_to_string(&snapshot_file).await {
            // Try to load as a tuple (db, ttl_db)
            if let Ok((loaded_db, loaded_ttl)) = serde_json::from_str::<(
                HashMap<String, String>,
                HashMap<String, u64>,
            )>(&snapshot_data)
            {
                db = loaded_db;
                ttl_db = loaded_ttl;
            } else if let Ok(loaded_db) =
                serde_json::from_str::<HashMap<String, String>>(&snapshot_data)
            {
                // Fallback: If snapshot only contains db (backward compatibility)
                db = loaded_db;
            }

            // Rebuild the expiry heap from the loaded TTLs
            for (k, v) in &ttl_db {
                expiry_heap.push(Reverse((*v, k.clone())));
            }
        }

        let wal_file = format!("wal_{}.log", shard_id);
        // --- 2. Replay WAL ---
        if let Ok(wal_content) = tokio::fs::read_to_string(&wal_file).await {
            for line in wal_content.lines() {
                if let Some(cmd) = parse_command(line) {
                    apply_db(&mut db, &mut ttl_db, &mut expiry_heap, cmd);
                }
            }
        }

        loop {
            tokio::select! {
                // --- 3. Expiration Task ---
                _ = cleanup_interval.tick() => {
                    let now = now_ms();
                    let mut expired_count = 0;
                    // Limit how many keys we scan per tick to avoid blocking
                    while expired_count < 200 {
                        match expiry_heap.peek() {
                            Some(Reverse((exp, _))) if *exp <= now => {
                                let Reverse((exp2, key)) = expiry_heap.pop().unwrap();
                                // Only delete if the TTL in heap matches the actual DB (handles overwrites)
                                if ttl_db.get(&key) == Some(&exp2) {
                                    db.remove(&key);
                                    ttl_db.remove(&key);
                                }
                                expired_count += 1;
                            }
                            _ => break,
                        }
                    }
                }

                // --- 4. Snapshot Task ---
                _ = snapshot_interval.tick() => {
                    let db_snapshot = db.clone();
                    let ttl_snapshot = ttl_db.clone();
                    let wal_tx_clone = wal_tx.clone();

                    task::spawn_blocking(move || {
                        let combined_state = (db_snapshot, ttl_snapshot);
                        save_snapshot(shard_id,&combined_state.0, &combined_state.1);
                    }).await.unwrap();


                    let _ = wal_tx_clone.send(WalCommand::Truncate).await;
                }

                Some(cmd) = cmd_rx.recv() => {
                    let now = now_ms();
                    match cmd {
                        Command::Set { key, value, resp } => {
                            let _ = wal_tx.send(WalCommand::Write(format!("SET {} {}\n", key, value))).await;
                            db.insert(key, value);
                            let _ = resp.send("OK\n".to_string());
                        }
                        Command::SetEx { key, value, ttl, resp } => {
                            let expiry = now + ttl * 1000;
                            let _ = wal_tx.send(WalCommand::Write(format!("SETEX {} {} {}\n", key, value, ttl))).await;
                            db.insert(key.clone(), value);
                            ttl_db.insert(key.clone(), expiry);
                            expiry_heap.push(Reverse((expiry, key)));
                            let _ = resp.send("OK\n".to_string());
                        }
                        Command::Get { key, resp } => {
                            // Lazy expiration check on Access
                            if let Some(&expiry) = ttl_db.get(&key) {
                                if expiry <= now {
                                    db.remove(&key);
                                    ttl_db.remove(&key);
                                }
                            }
                            let value = db.get(&key).cloned().unwrap_or_else(|| "nil".into());
                            let _ = resp.send(value);
                        }
                        Command::Expire { key, ttl, resp } => {
                            if db.contains_key(&key) {
                                let expiry = now + ttl * 1000;
                                let _ = wal_tx.send(WalCommand::Write(format!("EXPIRE {} {}\n", key, ttl))).await;
                                ttl_db.insert(key.clone(), expiry);
                                expiry_heap.push(Reverse((expiry, key)));
                                let _ = resp.send("1".into());
                            } else {
                                let _ = resp.send("0".into());
                            }
                        }
                        Command::Del { key, resp } => {
                            let _ = wal_tx.send(WalCommand::Write(format!("DEL {}\n", key))).await;
                            let removed = db.remove(&key).is_some();
                            ttl_db.remove(&key);
                            let _ = resp.send(if removed { "1".into() } else { "0".into() });
                        }
                        Command::Ex { key, resp } => {
                            let exists = db.contains_key(&key);
                            let _ = resp.send(if exists { "1".into() } else { "0".into() });
                        }
                        Command::Ttl { key, resp } => {
                            if !db.contains_key(&key) {
                                let _ = resp.send("-2".into());
                            } else if let Some(&expiry) = ttl_db.get(&key) {
                                if expiry <= now {
                                    db.remove(&key);
                                    ttl_db.remove(&key);
                                    let _ = resp.send("-2".into());
                                } else {
                                    let ttl = (expiry - now) / 1000;
                                    let _ = resp.send(ttl.to_string());
                                }
                            } else {
                                let _ = resp.send("-1".into());
                            }
                        }
                    }
                }
            }
        }
    });
}
