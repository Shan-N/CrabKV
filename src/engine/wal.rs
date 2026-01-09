use std::cmp::Reverse;
use std::collections::HashMap;
use std::mem;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{task, time};

use super::{Command, WalCommand, apply_db, save_snapshot};
use crate::engine::apply::now_ms;
use crate::engine::command::WalEntry;
use crate::engine::ParsedCommand;

pub fn start_wal_task(shard_id: usize, mut wal_rx: Receiver<WalCommand>) {
    let filename = format!("wal_{}.log", shard_id);
    task::spawn(async move {
        let wal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .await
            .expect("Failed to open WAL");

        let mut writer = BufWriter::with_capacity(64 * 1024, wal);
        let mut buffer = Vec::with_capacity(128 * 1024);
        let mut sync_interval = time::interval(Duration::from_millis(5));

        loop {
            tokio::select! {
                _ = sync_interval.tick() => {
                    if !buffer.is_empty() {
                        let _ = writer.write_all(&buffer).await;
                        buffer.clear();
                    }
                }

                entry = wal_rx.recv() => {
                    match entry {
                        Some(WalCommand::Write(s)) => {
                            buffer.extend_from_slice(&s);

                            if buffer.len() >= 128 * 1024 {
                                let _ = writer.write_all(&buffer).await;
                                let _ = writer.flush().await;
                                buffer.clear();
                            }
                        }
                        Some(WalCommand::Truncate) => {
                            // 1. Flush BufWriter internal buffer to file
                            writer.flush().await.unwrap();

                            // 2. Write our extra WAL batch buffer directly to file
                            if !buffer.is_empty() {
                                let file = writer.get_mut();
                                file.write_all(&buffer).await.unwrap();
                                file.flush().await.unwrap();
                                buffer.clear();
                            }

                            // 3. Truncate file to zero
                            let file = writer.get_mut();
                            file.set_len(0).await.unwrap();
                            file.seek(std::io::SeekFrom::Start(0)).await.unwrap();

                            // 4. Final flush to commit metadata
                            file.flush().await.unwrap();

                        }
                        None => break,
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

        if let Ok(snapshot_data) = tokio::fs::read_to_string(&snapshot_file).await {
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

                db = loaded_db;
            }

            for (k, v) in &ttl_db {
                expiry_heap.push(Reverse((*v, k.clone())));
            }
        }

        let wal_file = format!("wal_{}.log", shard_id);
        if let Ok(wal_content) = tokio::fs::read(&wal_file).await {
            let mut slice = wal_content.as_slice();
            
            while !slice.is_empty() {
                match bincode::deserialize_from::<_,WalEntry>(&mut slice) {
                    Ok(entry) => {
                        match entry {
                            WalEntry::Set { key, value } => {
                                apply_db(&mut db, &mut ttl_db, &mut expiry_heap, ParsedCommand::Set { key, value });
                            },
                            WalEntry::SetEx { key, value, ttl } => {
                                apply_db(&mut db, &mut ttl_db, &mut expiry_heap, ParsedCommand::SetEx { key, value, ttl });
                            },
                            WalEntry::Del { key } => {
                                apply_db(&mut db, &mut ttl_db, &mut expiry_heap, ParsedCommand::Del { key });
                            },
                            WalEntry::Expire { key, ttl } => {
                                apply_db(&mut db, &mut ttl_db, &mut expiry_heap, ParsedCommand::Expire { key, ttl });
                            },
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        let mut encoded = Vec::with_capacity(128);

        loop {
            tokio::select! {
                _ = cleanup_interval.tick() => {
                    let now = now_ms();
                    let mut expired_count = 0;
                    while expired_count < 200 {
                        match expiry_heap.peek() {
                            Some(Reverse((exp, _))) if *exp <= now => {
                                let Reverse((exp2, key)) = expiry_heap.pop().unwrap();
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

                            let entry = WalEntry::Set { key: key.clone(), value: value.clone() };
                            // let mut encoded = Vec::with_capacity(64);
                            encoded.clear();
                            bincode::serialize_into(&mut encoded, &entry).unwrap();
                            // let log = format!("SET {} {}\n", key, value);
                            let _ = wal_tx.send(WalCommand::Write(mem::take(&mut encoded))).await;
                            db.insert(key, value);
                            let _ = resp.send("OK\n".to_string());
                        }
                        Command::SetEx { key, value, ttl, resp } => {
                            let entry = WalEntry::SetEx { key: key.clone(), value: value.clone(), ttl };
                            // let mut encoded = Vec::with_capacity(64);
                            encoded.clear();
                            bincode::serialize_into(&mut encoded, &entry).unwrap();
                            // let log = format!("SETEX {} {} {}\n", key, value, ttl);
                            let expiry = now + ttl * 1000;
                            let _ = wal_tx.send(WalCommand::Write(mem::take(&mut encoded))).await;
                            db.insert(key.clone(), value);
                            ttl_db.insert(key.clone(), expiry);
                            expiry_heap.push(Reverse((expiry, key)));
                            let _ = resp.send("OK\n".to_string());
                        }
                        Command::Get { key, resp } => {
                            if let Some(&expiry) = ttl_db.get(&key) {
                                if expiry <= now {
                                    db.remove(&key);
                                    ttl_db.remove(&key);
                                }
                            }
                            let value = db.get(&key).cloned().unwrap_or_else(|| "nil\n".into());
                            let _ = resp.send(value);
                        }
                        Command::Expire { key, ttl, resp } => {
                            // let log = format!("EXPIRE {} {}\n", key, ttl);
                            let entry = WalEntry::Expire { key: key.clone(), ttl };
                            // let mut encoded = Vec::with_capacity(64);
                            encoded.clear();
                            bincode::serialize_into(&mut encoded, &entry).unwrap();
                            if db.contains_key(&key) {
                                let expiry = now + ttl * 1000;
                                let _ = wal_tx.send(WalCommand::Write(mem::take(&mut encoded))).await;
                                ttl_db.insert(key.clone(), expiry);
                                expiry_heap.push(Reverse((expiry, key)));
                                let _ = resp.send("1".into());
                            } else {
                                let _ = resp.send("0".into());
                            }
                        }
                        Command::Del { key, resp } => {
                            // let log = format!("DEL {}\n", key);
                            let entry = WalEntry::Del { key: key.clone() };
                            // let mut encoded = Vec::with_capacity(64);
                            encoded.clear();
                            bincode::serialize_into(&mut encoded, &entry).unwrap();
                            let _ = wal_tx.send(WalCommand::Write(mem::take(&mut encoded))).await;
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
                        Command::Ping { resp } => {
                            let _ = resp.send("PONG\n".to_string());
                        }
                        
                    }
                }
            }
        }
    });
}
