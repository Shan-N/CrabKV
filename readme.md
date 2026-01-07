# 🦀 CrabKV: Based Async Store

Yo, this is **CrabKV**. We built this in **Rust** because C++ is legacy bloat and we don't do memory leaks.

It’s an async, persistent key-value store running on **Tokio**. It’s fully non-blocking, memory safe, and honestly just built different.

**Status:** Goated. 🐐

## 🗿 Why It’s Based (Features)

* **Async Core ⚡**: Built on the Actor Model because mutex locks are for boomers. We use channels. Pure throughput. No cap.
* **Persistent (WAL) 📝**: We log every write to `wal.log` instantly. Server crash? Skill issue. We recover instantly.
* **Snapshots 📸**: We dump the DB state to `snapshot.json` so cold starts are instant.
* **TTL (Ghost) 👻**: Keys expire automatically. Clean up your garbage. (`SETEX`, `EXPIRE`).
* **Protocol 🤝**: Simple TCP text protocol. `netcat` friendly.

## 📂 The Stack (Structure)

Clean architecture only.

```text
├── src
│   ├── engine          # Core database logic (State, WAL, Snapshots)
│   │   ├── apply.rs    # Command application logic
│   │   ├── command.rs  # Command definitions (Set, Get, Del)
│   │   ├── parser.rs   # Inbound command parsing
│   │   ├── snapshot.rs # State serialization
│   │   └── wal.rs      # Write-Ahead Log management
│   ├── server          # Networking layer
│   │   ├── connection.rs # TCP stream handler
│   │   └── mod.rs
│   └── main.rs         # Entry point & Runtime setup
├── snapshot.json       # Persisted DB state
└── wal.log             # Append-only operation log

```

## 🚀 Deployment

### Prereqs

* **Rust**: If you aren't on the latest stable toolchain, what are you even doing?

### Run It

Clone it, ship it.

```bash
# Clone
git clone https://github.com/Shan-N/crabkv.git

# Release Build (Debug is for weakness)
cargo run --release

```

Server binds to `127.0.0.1:3000`. We live.

### Usage

Hit it with `nc` or telnet.

```bash
$ nc localhost 3000
SET performance yes
OK
GET performance
yes
SETEX temp_data 10 gone
OK
TTL temp_data
9

```

## 🛠 Command Tier List

| Command | Usage | Description |
| --- | --- | --- |
| **SET** | `SET k v` | Lock it in. 🔒 |
| **GET** | `GET k` | Fetch the alpha. |
| **DEL** | `DEL k` | Nuke it. 💥 |
| **SETEX** | `SETEX k t v` | Ephemeral storage. |
| **TTL** | `TTL k` | Countdown. |

## 🗺 Grindset (Roadmap)

We are single-threaded right now (MVP vibes), but the scale-up is coming.

* [ ] **Sharding**: `todo!("add sharding")` — **CRITICAL**. We need to split the keyspace to saturate all cores. We scaling to the moon. 🚀
* [ ] **Binary Protocol**: Text parsing is CPU waste. We switching to binary.
* [ ] **Client Lib**: Native Rust crate incoming.

## 📄 License

MIT. Fork it. Ship it.

---

**Built with 🦀 and Sleep Deprivation by Shannn.**