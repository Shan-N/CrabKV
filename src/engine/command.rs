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
}

pub enum WalCommand {
    Write(String),
    Truncate,
}
