use crate::engine::{Command, ParsedCommand, parse_command};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Sender,
    sync::oneshot,
};

pub async fn handle_connection(mut socket: TcpStream, addr: SocketAddr, cmd_tx: Sender<Command>) {
    let mut buf = Vec::new();
    println!("New connection from {}", addr);
    loop {
        let mut temp = [0u8; 1024];
        let n = match socket.read(&mut temp).await {
            Ok(0) => return,
            Ok(n) => n,
            Err(_) => return,
        };

        buf.extend_from_slice(&temp[..n]);

        while let Some(idx) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.drain(..=idx).collect::<Vec<_>>();
            let input = String::from_utf8_lossy(&line);
            if let Some(parsed) = parse_command(input.trim()) {
                let (resp_tx, resp_rx) = oneshot::channel();
                let cmd = match parsed {
                    ParsedCommand::Set { key, value } => Command::Set {
                        key,
                        value,
                        resp: resp_tx,
                    },
                    ParsedCommand::SetEx { key, value, ttl } => Command::SetEx {
                        key,
                        value,
                        ttl,
                        resp: resp_tx,
                    },
                    ParsedCommand::Get { key } => Command::Get { key, resp: resp_tx },
                    ParsedCommand::Del { key } => Command::Del { key, resp: resp_tx },
                    ParsedCommand::Expire { key, ttl } => Command::Expire {
                        key,
                        ttl,
                        resp: resp_tx,
                    },
                    ParsedCommand::Ttl { key } => Command::Ttl { key, resp: resp_tx },
                    ParsedCommand::Ex { key } => Command::Ex { key, resp: resp_tx },
                };

                cmd_tx.send(cmd).await.unwrap();
                let response = resp_rx.await.unwrap();
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        }
    }
}
