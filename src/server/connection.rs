use crate::{
    engine::{Command, ParsedCommand, parse_command},
    shard_engine::router::ShardRouter,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
    sync::oneshot,
};

pub async fn handle_connection(socket: TcpStream, _addr: SocketAddr, router: Arc<ShardRouter>) {
    let mut stream = BufWriter::with_capacity(8 * 1024, socket);
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut temp = [0u8; 8 * 1024]; 

    loop {
        let n = match stream.get_mut().read(&mut temp).await {
            Ok(0) => return, 
            Ok(n) => n,
            Err(_) => return, 
        };

        buf.extend_from_slice(&temp[..n]);

        let mut commands_processed = 0;

        while let Some(idx) = buf.iter().position(|&b| b == b'\n') {
            let line_slice = &buf[..idx];
            
            if let Ok(input_str) = std::str::from_utf8(line_slice) {
                if let Some(parsed) = parse_command(input_str.trim()) {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    
                    let cmd = match parsed {
                        ParsedCommand::Set { key, value } => Command::Set { key, value, resp: resp_tx },
                        ParsedCommand::SetEx { key, value, ttl } => Command::SetEx { key, value, ttl, resp: resp_tx },
                        ParsedCommand::Get { key } => Command::Get { key, resp: resp_tx },
                        ParsedCommand::Del { key } => Command::Del { key, resp: resp_tx },
                        ParsedCommand::Expire { key, ttl } => Command::Expire { key, ttl, resp: resp_tx },
                        ParsedCommand::Ttl { key } => Command::Ttl { key, resp: resp_tx },
                        ParsedCommand::Ex { key } => Command::Ex { key, resp: resp_tx },
                        ParsedCommand::Ping {} => Command::Ping { resp: resp_tx },

                    };

                    router.route(cmd).await;
                    
                    match resp_rx.await {
                        Ok(response) => {
                            if stream.write_all(response.as_bytes()).await.is_err() {
                                return; 
                            }
                            commands_processed += 1;
                        }
                        Err(_) => return,
                    }
                }
            }
            
            buf.drain(..=idx);
        }

        if commands_processed > 0 {
            if stream.flush().await.is_err() {
                return;
            }
        }
    }
}