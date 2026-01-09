use std::collections::HashMap;
use std::fs::{File, rename};
use std::io::{BufWriter, Write};
use bincode;

pub fn save_snapshot(shard_id: usize, db: &HashMap<String, String>, ttl_db: &HashMap<String, u64>) {
    let tmp_path = format!("snapshot_{}.bin.tmp", shard_id);
    let final_path = format!("snapshot_{}.bin", shard_id);

    let file = File::create(&tmp_path).unwrap();
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    bincode::serialize_into(&mut writer, &(db, ttl_db)).unwrap();


    writer.flush().unwrap();

    rename(tmp_path, final_path).unwrap();
}
