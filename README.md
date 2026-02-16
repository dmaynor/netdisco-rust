# Netdisco-Rust

[![Tests](https://img.shields.io/badge/tests-203%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)]()
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue)]()

A high-performance Rust port of [App::Netdisco](https://metacpan.org/pod/App::Netdisco), the open-source network management tool for L2/L3 networks.

## What is Netdisco?

Netdisco is a network management tool that:

- **Discovers** network devices via SNMP (v1/v2c/v3)
- **Collects** MAC address tables (L2 forwarding) and ARP tables (L3 mappings)
- **Tracks** which hosts are connected to which switch ports
- **Provides** a web interface for searching devices, nodes, and port information
- **Manages** port configuration (admin up/down, VLAN changes, descriptions)
- **Schedules** background polling jobs on configurable intervals

This Rust port delivers the same functionality with improved performance, memory safety, and modern tooling.

## Quick Start

```bash
# Clone and build
git clone https://github.com/dmaynor/netdisco-rust.git
cd netdisco-rust
cargo build --release

# Set up the database
export NETDISCO_DB_HOST=localhost
export NETDISCO_DB_NAME=netdisco
export NETDISCO_DB_USER=netdisco
export NETDISCO_DB_PASS=your_password
cargo run --bin netdisco-deploy

# Start the web server (port 5000)
cargo run --bin netdisco-web

# Start the backend daemon
cargo run --bin netdisco-backend
```

## Architecture

```
netdisco-rust/
├── src/
│   ├── lib.rs              # Library root (re-exports all modules)
│   ├── main.rs             # Default binary (help text)
│   ├── bin/
│   │   ├── web.rs          # Web server binary (Actix-web on port 5000)
│   │   ├── backend.rs      # Backend job processing daemon
│   │   ├── do_cmd.rs       # CLI tool for ad-hoc operations
│   │   └── deploy.rs       # Database schema migration tool
│   ├── config/             # YAML configuration loading with env overrides
│   │   ├── mod.rs          # Layered config: defaults → YAML → env vars
│   │   └── settings.rs     # 80+ configuration fields with defaults
│   ├── db/                 # PostgreSQL layer (async via SQLx)
│   │   ├── pool.rs         # Connection pool management
│   │   ├── queries.rs      # Typed SQL queries
│   │   └── migrate.rs      # Schema migration runner
│   ├── models/             # 30+ data models mapping to PostgreSQL tables
│   │   ├── device.rs       # Network device (switch, router, AP)
│   │   ├── node.rs         # MAC addresses seen on switch ports
│   │   ├── admin.rs        # Backend job queue
│   │   ├── user.rs         # Authentication & authorization
│   │   └── ...             # Ports, VLANs, modules, power, topology, etc.
│   ├── snmp/               # SNMP client (raw UDP, BER encoding)
│   │   ├── client.rs       # GET/GETNEXT/GETBULK/WALK operations
│   │   └── oids.rs         # 50+ standard OID definitions
│   ├── web/                # HTTP layer
│   │   ├── routes.rs       # URL → handler mapping
│   │   ├── handlers.rs     # Page rendering handlers
│   │   ├── api.rs          # REST API v1 (JSON)
│   │   └── auth.rs         # Session management & authentication
│   ├── backend/            # Background job processing
│   │   ├── mod.rs          # Job dispatcher
│   │   └── scheduler.rs    # Cron-based job scheduling
│   ├── worker/             # Task execution plugins
│   │   ├── discover.rs     # SNMP device discovery
│   │   ├── macsuck.rs      # MAC address table collection
│   │   ├── arpnip.rs       # ARP table collection
│   │   ├── expire.rs       # Old record expiration
│   │   ├── nbtstat.rs      # NetBIOS name resolution
│   │   └── portcontrol.rs  # Port admin (up/down/vlan/name)
│   └── util/               # Shared utilities
│       ├── mod.rs          # MAC formatting, uptime, ACL matching
│       ├── dns.rs          # DNS resolution helpers
│       ├── net.rs          # Ping, private IP detection
│       └── permission.rs   # Device ACL (discover_only, discover_no)
├── tests/                  # Comprehensive test suite (203 tests)
│   ├── test_suite.rs       # Main test harness
│   ├── unit/               # Unit tests (152 tests)
│   ├── integration/        # Integration tests (37 tests)
│   └── e2e/                # End-to-end workflow tests (14 tests)
├── migrations/             # PostgreSQL schema migrations
├── Cargo.toml              # Dependencies and build configuration
└── docs/                   # In-depth documentation
```

## Binaries

| Binary | Description | Default Port |
|--------|-------------|-------------|
| `netdisco` | Shows help text and available commands | — |
| `netdisco-web` | Web frontend server with REST API | 5000 |
| `netdisco-backend` | Background job processing daemon | — |
| `netdisco-do` | CLI for ad-hoc operations (discover, macsuck, etc.) | — |
| `netdisco-deploy` | Database schema migration tool | — |

## Technology Stack

| Component | Crate | Purpose |
|-----------|-------|---------|
| Web Server | `actix-web 4` | HTTP server, routing, middleware |
| Database | `sqlx 0.7` | Async PostgreSQL with compile-time validation |
| Async Runtime | `tokio 1` | Event loop for backend workers |
| Serialization | `serde` + `serde_yaml` | Config files and JSON API |
| CLI | `clap 4` | Argument parsing with derive macros |
| Templates | `tera 1` | Jinja2-style HTML templating |
| Logging | `tracing` | Structured, leveled logging |
| DNS | `trust-dns-resolver` | Async DNS resolution |
| Auth | `bcrypt` + `actix-session` | Password hashing and cookie sessions |

## Testing

```bash
# Run the full test suite (203 tests)
cargo test --test test_suite

# Run only unit tests
cargo test --test test_suite unit::

# Run only integration tests
cargo test --test test_suite integration::

# Run only end-to-end tests
cargo test --test test_suite e2e::
```

### Test Coverage

| Category | Tests | Coverage |
|----------|-------|----------|
| **Unit — Config** | 36 | Defaults (13 groups), YAML overrides, env var overrides, connection strings |
| **Unit — Models** | 40 | Device layers/display, Node OUI extraction, User auth, Admin jobs, serialization |
| **Unit — Utilities** | 49 | MAC formatting (10 formats), uptime (9), ACL matching (15), private IP (8), DNS |
| **Unit — SNMP** | 27 | Versions, credentials, client creation, OID constants, data structures |
| **Integration — Web API** | 12 | Routes, JSON responses, content-type, HTTP methods, 404/400 handling |
| **Integration — Handlers** | 11 | Redirects, sessions, IP validation, search detection, login |
| **Integration — Backend** | 14 | Job dispatch, port control, worker calc, expiration |
| **E2E — Workflows** | 14 | Discovery pipeline, job lifecycle, full API workflow, MAC processing |

## Requirements

- **Rust** 1.70+ (2021 edition)
- **PostgreSQL** 13+
- Network access to SNMP-enabled devices (for discovery)

## License

BSD-3-Clause — same as the original Netdisco project.

## See Also

- [Original Netdisco (Perl)](https://github.com/netdisco/netdisco)
- [Netdisco-Zig](https://github.com/dmaynor/netdisco-zig) — The Zig port
- [Documentation](docs/) — In-depth usage guide
