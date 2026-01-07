pub mod connection;

use crate::engine::Command;
use connection::handle_connection;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

pub async fn run(listener: TcpListener, cmd_tx: Sender<Command>) {
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(socket, addr, cmd_tx.clone()));
    }
}
