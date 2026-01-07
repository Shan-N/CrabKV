use tokio::{net::TcpListener, sync::mpsc};

mod engine;
mod server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server listening on port 3000");
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (wal_tx, wal_rx) = mpsc::channel(100);

    engine::start_engine(cmd_rx, wal_tx.clone());
    engine::start_wal_task(wal_rx);

    server::run(listener, cmd_tx).await;

    // todo!("add sharding support");
}
