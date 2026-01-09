use tokio::sync::oneshot;

pub enum Command {
    Set {
        key: String,
        value: String,
        resp: oneshot::Sender<String>,
    },
    SetEx {
        key: String,
        value: String,
        ttl: u64,
        resp: oneshot::Sender<String>,
    },
    Get {
        key: String,
        resp: oneshot::Sender<String>,
    },
    Del {
        key: String,
        resp: oneshot::Sender<String>,
    },
    Ex {
        key: String,
        resp: oneshot::Sender<String>,
    },
    Expire {
        key: String,
        ttl: u64,
        resp: oneshot::Sender<String>,
    },
    Ttl {
        key: String,
        resp: oneshot::Sender<String>,
    },
    Ping {
        resp: oneshot::Sender<String>,
    },
}

pub enum ParsedCommand {
    Set {
        key: String,
        value: String,
    },
    SetEx {
        key: String,
        value: String,
        ttl: u64,
    },
    Get {
        key: String,
    },
    Del {
        key: String,
    },
    Ex {
        key: String,
    },
    Expire {
        key: String,
        ttl: u64,
    },
    Ttl {
        key: String,
    },
    Ping
}

impl Command {
    pub fn primary_key(&self) -> &str {
        match self {
            Command::Set { key, .. } => key,
            Command::Get { key, .. } => key,
            Command::Del { key, .. } => key,
            Command::SetEx { key, .. } => key,
            Command::Expire { key, .. } => key,
            Command::Ex { key, .. } => key,
            Command::Ttl { key, .. } => key,
            Command::Ping { .. } => "",
        }
    }
}

pub enum WalCommand {
    Write(String),
    Truncate,
}
