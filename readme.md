# 🦀 CrabKV

**CrabKV** is a high-performance, asynchronous, persistent key-value store written in Rust. It is designed from the ground up using **Tokio** to handle high concurrency, featuring Write-Ahead Logging (WAL) for durability and periodic snapshots for fast recovery.

## ⚡ Features

* **Async Core**: Built on the **Tokio** runtime using an actor-based architecture (MPSC channels) to eliminate mutex contention on the hot path.
* **Durability**:
* **WAL (Write-Ahead Log)**: Every write operation is appended to `wal.log` immediately.
* **Snapshots**: State is periodically flushed to `snapshot.json` to speed up restarts.


* **Key Expiration**: Native support for TTL (Time-To-Live) via `SETEX` and `EXPIRE`.
* **Protocol**: Simple text-based TCP protocol (Redis-compatible for basic commands).

## 📂 Project Structure

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

## 🚀 Getting Started

### Prerequisites

* Rust (latest stable)
* Cargo

### Installation & Run

```bash
# Clone the repository
git clone https://github.com/yourusername/crabkv.git

# Run the server in release mode
cargo run --release

```

The server will start listening on `127.0.0.1:3000`.

### Basic Usage

You can interact with CrabKV using `netcat`, `telnet`, or `redis-cli` (for supported text commands).

```bash
$ nc localhost 3000
SET user:1 rustacean
OK
GET user:1
rustacean
SETEX session active 10
OK
TTL session
9

```

## 🛠 Supported Commands

| Command | Syntax | Description |
| --- | --- | --- |
| **SET** | `SET key value` | Store a string value. |
| **GET** | `GET key` | Retrieve a value. |
| **DEL** | `DEL key` | Delete a key. |
| **SETEX** | `SETEX key val ttl` | Set key with expiration (seconds). |
| **EXPIRE** | `EXPIRE key ttl` | Set a timeout on an existing key. |
| **TTL** | `TTL key` | Get remaining time to live. |

## 🗺 Roadmap

The current version runs a single engine instance. The immediate next step is scaling:

* [ ] **Sharding**: `todo!("add sharding")` — Implement data partitioning to split the keyspace across multiple engine actors to utilize multiple cores and reduce lock contention.
* [ ] **Binary Protocol**: Move to a binary protocol for lower parsing overhead (RESP).
* [ ] **Client Lib**: Develop a dedicated Rust client crate for CrabKV.

## 📄 License

This project is licensed under the MIT License.