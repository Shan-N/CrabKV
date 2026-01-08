pub mod connection;

use std::sync::Arc;

use crate::shard_engine::router::ShardRouter;
use connection::handle_connection;
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener, router: Arc<ShardRouter>) {
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(socket, addr, router.clone()));
    }
}
