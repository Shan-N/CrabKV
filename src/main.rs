use std::sync::Arc;

use tokio::net::TcpListener;

use crate::shard_engine::router::ShardRouter;

mod engine;
mod server;
mod shard_engine;

const NUM_SHARDS: usize = 16;

#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server listening on port 3000");
    let shards = shard_engine::engine::spawn_shards(NUM_SHARDS);
    let router = Arc::new(ShardRouter::new(shards));

    server::run(listener, router.clone()).await;
}
