# Netdisco-Rust

A Rust port of the [App::Netdisco](https://metacpan.org/pod/App::Netdisco) network management tool.

## Overview

Netdisco is a network management tool for L2/L3 networks. It discovers devices via SNMP, collects MAC and ARP tables, and provides a web interface for searching and managing the network.

This port reimplements the entire application in Rust using:

- **Actix-web** - HTTP server and REST API
- **SQLx** - Async PostgreSQL client with compile-time query validation
- **Tokio** - Async runtime for the backend workers
- **Serde** - Configuration and data serialization
- **clap** - CLI argument parsing

## Project Structure

```
netdisco-rust/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # Default binary
│   ├── bin/
│   │   ├── web.rs          # Web server binary
│   │   ├── backend.rs      # Backend daemon binary
│   │   ├── do_cmd.rs       # CLI tool (netdisco-do)
│   │   └── deploy.rs       # Database deployment tool
│   ├── config/             # Configuration loading (YAML)
│   ├── db/                 # Database layer (SQLx, PostgreSQL)
│   ├── models/             # 30+ data models (Device, Node, Port, etc.)
│   ├── snmp/               # SNMP client (BER encoding, OID definitions)
│   ├── web/                # HTTP handlers, routes, REST API, auth
│   ├── backend/            # Job processing daemon, scheduler
│   ├── worker/             # Task plugins (discover, macsuck, arpnip, etc.)
│   └── util/               # DNS, MAC formatting, ACL, networking
├── migrations/             # SQL schema migrations
├── Cargo.toml              # Dependencies and build config
└── README.md
```

## Binaries

| Binary | Description |
|--------|-------------|
| `netdisco-web` | Web frontend server (port 5000) |
| `netdisco-backend` | Background job processing daemon |
| `netdisco-do` | CLI for ad-hoc operations |
| `netdisco-deploy` | Database schema migration tool |

## Building

```bash
# Build all binaries
cargo build --release

# Run the web server
cargo run --bin netdisco-web

# Run the backend daemon
cargo run --bin netdisco-backend

# Use the CLI
cargo run --bin netdisco-do -- discover --device 192.168.1.1
cargo run --bin netdisco-do -- stats

# Deploy the database
cargo run --bin netdisco-deploy
```

## Configuration

Configuration uses the same YAML format as the original Perl Netdisco:

1. `config.yml` - Default configuration
2. `environments/deployment.yml` - Local overrides
3. Environment variables (`NETDISCO_DB_HOST`, etc.)

## Requirements

- Rust 1.70+
- PostgreSQL 13+
- Network access to SNMP-enabled devices
