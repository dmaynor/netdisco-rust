# Netdisco-Rust Documentation

Comprehensive guide to installing, configuring, and operating Netdisco-Rust.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Database Setup](#database-setup)
- [Web Server](#web-server)
- [Backend Daemon](#backend-daemon)
- [CLI Operations](#cli-operations)
- [REST API](#rest-api)
- [SNMP Discovery](#snmp-discovery)
- [Authentication & Authorization](#authentication--authorization)
- [Job Queue](#job-queue)
- [Port Control](#port-control)
- [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Rust | 1.70+ | Compiler and toolchain |
| PostgreSQL | 13+ | Database backend |
| SNMP | v1/v2c/v3 | Device polling (network devices must support SNMP) |

### Building from Source

```bash
git clone https://github.com/dmaynor/netdisco-rust.git
cd netdisco-rust

# Debug build (fast compile, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# The binaries will be in target/release/
ls target/release/netdisco*
```

### Verifying the Installation

```bash
# Show version and available commands
cargo run --bin netdisco

# Run the test suite to verify everything works
cargo test --test test_suite
# Expected: 203 tests passing
```

---

## Configuration

Netdisco uses a **layered configuration system** with the following precedence (highest wins):

1. **Environment variables** — `NETDISCO_DB_HOST`, `NETDISCO_DB_NAME`, etc.
2. **Deployment overlay** — `environments/deployment.yml`
3. **Base config** — `config.yml`
4. **Built-in defaults** — Hardcoded in `src/config/settings.rs`

### Configuration File Format

Create a `config.yml` in your working directory or config path:

```yaml
# Database connection
database:
  name: netdisco
  host: localhost
  user: netdisco
  pass: your_password_here
  port: 5432

# SNMP settings
community:
  - public
  - my_community
snmpver: 2
snmptimeout: 3000000    # microseconds (3 seconds)
snmpretries: 2

# Web server
web_home: /inventory
branding_text: "My Network"
table_pagesize: 25
no_auth: false

# Discovery behavior
discover_neighbors: true
discover_routed_neighbors: true
discover_waps: true
discover_phones: false

# Data retention (days)
expire_devices: 60
expire_nodes: 90
expire_jobs: 14

# Workers
workers:
  count: 4
  timeout: 600
  sleep: 1

# Scheduling (cron format)
schedule:
  discover: "0 */6 * * *"    # Every 6 hours
  macsuck:  "0 */2 * * *"    # Every 2 hours
  arpnip:   "0 */2 * * *"    # Every 2 hours
  expire:   "0 4 * * *"      # Daily at 4 AM
```

### Environment Variable Overrides

Environment variables take the **highest precedence** and are useful for containerized deployments:

| Variable | Description | Example |
|----------|-------------|---------|
| `NETDISCO_DB_NAME` | Database name | `netdisco` |
| `NETDISCO_DB_HOST` | Database hostname | `db.example.com` |
| `NETDISCO_DB_USER` | Database username | `netdisco` |
| `NETDISCO_DB_PASS` | Database password | `s3cret` |
| `NETDISCO_DB_PORT` | Database port | `5432` |
| `NETDISCO_RO_COMMUNITY` | SNMP read communities (comma-separated) | `public,corp_ro` |
| `NETDISCO_RW_COMMUNITY` | SNMP write communities | `private` |
| `NETDISCO_SNMPVER` | Default SNMP version | `2` |
| `NETDISCO_DOMAIN` | Domain suffixes to strip (comma-separated) | `.example.com,.corp` |
| `NETDISCO_NO_AUTH` | Disable authentication | `true` |
| `NETDISCO_TRUST_REMOTE_USER` | Trust reverse proxy auth header | `true` |

### Deployment Overlay

For environment-specific settings, create `environments/deployment.yml`:

```yaml
# Override just what differs from config.yml
database:
  host: prod-db.internal
  pass: production_password

log: info
no_auth: false
```

---

## Database Setup

### Creating the Database

```bash
# Create the PostgreSQL database and user
sudo -u postgres psql <<EOF
CREATE USER netdisco WITH PASSWORD 'your_password';
CREATE DATABASE netdisco OWNER netdisco;
GRANT ALL PRIVILEGES ON DATABASE netdisco TO netdisco;
EOF
```

### Running Migrations

```bash
# Deploy the schema (creates all tables)
cargo run --bin netdisco-deploy
```

The schema includes the following tables:

| Table | Description |
|-------|-------------|
| `device` | Discovered network devices |
| `device_port` | Physical/logical interfaces |
| `device_ip` | IP addresses on device interfaces |
| `device_module` | Hardware modules (line cards, power supplies) |
| `device_vlan` | VLANs configured on devices |
| `device_port_vlan` | VLAN-to-port mappings |
| `device_port_power` | PoE status per port |
| `device_power` | Device power supply status |
| `node` | MAC addresses seen on switch ports |
| `node_ip` | IP-to-MAC mappings (from ARP/NDP) |
| `node_nbt` | NetBIOS names |
| `admin` | Job queue |
| `users` | User accounts and permissions |

---

## Web Server

### Starting the Web Server

```bash
# Default (port 5000)
cargo run --bin netdisco-web

# Custom port
NETDISCO_WEB_PORT=8080 cargo run --bin netdisco-web
```

### Web Interface Pages

| Path | Description |
|------|-------------|
| `/` | Redirects to `/inventory` |
| `/inventory` | Device inventory with search |
| `/device/<ip>` | Device detail page |
| `/search/device` | Device search |
| `/search/node` | Node (MAC/IP) search |
| `/admin/jobqueue` | Background job queue |
| `/login` | Authentication page |
| `/logout` | End session |

### Static Assets

Place static files (CSS, JavaScript, images) in a `public/` directory. The web server serves them automatically from `/static/`.

---

## Backend Daemon

The backend daemon processes queued jobs in the background.

### Starting the Backend

```bash
cargo run --bin netdisco-backend
```

### How It Works

1. The daemon polls the `admin` table for jobs with `status = 'queued'`
2. Jobs are dispatched to **worker plugins** based on their `action` field
3. Workers execute the task (e.g., SNMP discovery) and update the job status
4. Completed jobs are marked as `done` or `error`
5. The scheduler enqueues new jobs based on cron expressions

### Worker Types

| Action | Description | Requires Device? |
|--------|-------------|:----------------:|
| `discover` | SNMP system info, interfaces, neighbors | ✅ |
| `discoverall` | Discover all known devices | ❌ |
| `macsuck` | Collect MAC address table | ✅ |
| `macwalk` | Macsuck all L2 devices | ❌ |
| `arpnip` | Collect ARP table | ✅ |
| `arpwalk` | Arpnip all L3 devices | ❌ |
| `nbtstat` | NetBIOS name query | ✅ |
| `expire` | Clean up old records | ❌ |
| `portcontrol` | Port admin actions | ✅ |

---

## CLI Operations

The `netdisco-do` binary runs operations directly from the command line:

```bash
# Discover a single device
cargo run --bin netdisco-do -- discover --device 192.168.1.1

# Discover all known devices
cargo run --bin netdisco-do -- discoverall

# Collect MAC addresses from a device
cargo run --bin netdisco-do -- macsuck --device 10.0.0.1

# Collect ARP table from a device
cargo run --bin netdisco-do -- arpnip --device 10.0.0.1

# Run data expiration
cargo run --bin netdisco-do -- expire

# Show database statistics
cargo run --bin netdisco-do -- stats

# Port control
cargo run --bin netdisco-do -- portcontrol --device 10.0.0.1 --port Gi0/1 --action down
```

---

## REST API

The REST API is available at `/api/v1/` when the web server is running.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/object/device` | List all devices |
| `GET` | `/api/v1/object/device/<ip>` | Get a specific device |
| `GET` | `/api/v1/search/device?q=<query>` | Search devices |
| `GET` | `/api/v1/search/node?q=<query>` | Search nodes (MAC/IP) |
| `GET` | `/api/v1/queue` | List queued jobs |
| `POST` | `/api/v1/queue` | Enqueue a new job |

### Examples

```bash
# List all devices
curl http://localhost:5000/api/v1/object/device

# Get a specific device
curl http://localhost:5000/api/v1/object/device/192.168.1.1

# Search for a MAC address
curl http://localhost:5000/api/v1/search/node?q=00:11:22:33:44:55

# Search for devices by name
curl http://localhost:5000/api/v1/search/device?q=core-switch

# Enqueue a discover job
curl -X POST http://localhost:5000/api/v1/queue \
  -H "Content-Type: application/json" \
  -d '{"action": "discover", "device": "192.168.1.1"}'

# Check job queue
curl http://localhost:5000/api/v1/queue
```

### Response Format

All API responses return JSON:

```json
{
  "devices": [
    {
      "ip": "192.168.1.1",
      "dns": "core-switch.example.com",
      "description": "Cisco IOS Software, C3750",
      "layers": "0110000",
      "ports": 48,
      "vendor": "cisco",
      "os": "IOS",
      "os_ver": "15.2(4)E",
      "uptime": 123456789,
      "last_discover": "2024-06-15T12:30:00Z"
    }
  ]
}
```

---

## SNMP Discovery

### What Gets Discovered

When you discover a device, Netdisco collects:

| Data | SNMP OID Group | Stored In |
|------|---------------|-----------|
| System description, contact, location | `1.3.6.1.2.1.1.*` (sysGroup) | `device` |
| Interface table (name, speed, status) | `1.3.6.1.2.1.2.2.1.*` (ifTable) | `device_port` |
| MAC address table | `1.3.6.1.2.1.17.4.3.1.*` (dot1dTpFdb) | `node` |
| ARP table | `1.3.6.1.2.1.4.22.1.2` (ipNetToMedia) | `node_ip` |
| LLDP neighbors | `1.0.8802.1.1.2.*` (lldpRemTable) | `device_port` (remote_*) |
| CDP neighbors | `1.3.6.1.4.1.9.9.23.*` (cdpCache) | `device_port` (remote_*) |
| Entity info (serial, model) | `1.3.6.1.2.1.47.1.1.1.1.*` | `device` |

### Device Layer Classification

The `layers` field is a 7-character string derived from the SNMP `sysServices` bitmask:

| Position | Layer | Meaning |
|----------|-------|---------|
| 1 | Physical (L1) | Repeater/hub |
| 2 | Data Link (L2) | Switch/bridge |
| 3 | Network (L3) | Router |
| 4 | Transport (L4) | — |
| 5-7 | Application (L5-7) | Host/server |

Example: `"0110000"` = L2 switch + L3 router

### ACL Filtering

Control which devices get discovered using ACL lists:

```yaml
# Only discover these networks
discover_only:
  - 10.0.0.0/8
  - 172.16.0.0/12

# Never discover these devices
discover_no:
  - 10.99.99.0/24

# Same pattern for macsuck and arpnip
macsuck_only: []
macsuck_no: []
arpnip_only: []
arpnip_no: []
```

---

## Authentication & Authorization

### User Roles

| Field | Description |
|-------|-------------|
| `admin` | Full access to all features including user management |
| `port_control` | Can change port settings (up/down, VLAN, name) |
| (default) | Read-only access to searches and device information |

### Auth Modes

| Mode | Config | Description |
|------|--------|-------------|
| Local | (default) | Username/password stored in database (bcrypt hashed) |
| No auth | `no_auth: true` | All access permitted without login |
| LDAP | `ldap: { ... }` | Authenticate against LDAP/Active Directory |
| RADIUS | `radius: { ... }` | Authenticate via RADIUS server |
| TACACS+ | `tacacs: { ... }` | Authenticate via TACACS+ server |
| Proxy | `trust_remote_user: true` | Trust `REMOTE_USER` header from reverse proxy |

### API Tokens

For programmatic access:

```bash
# Tokens are valid for api_token_lifetime seconds (default: 3600)
curl -X POST http://localhost:5000/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "secret"}'
```

---

## Job Queue

### Job Lifecycle

```
queued → running → done
                 → error
queued → deferred → queued (retry)
```

### Job Fields

| Field | Description |
|-------|-------------|
| `job` | Auto-incrementing job ID |
| `action` | Worker action (discover, macsuck, etc.) |
| `device` | Target device IP (optional for bulk actions) |
| `port` | Target port (for portcontrol) |
| `subaction` | Sub-operation (up, down, bounce, name, vlan) |
| `status` | Current state (queued/running/done/error/deferred) |
| `username` | User who submitted the job |
| `log` | Result/error message |
| `entered` | Timestamp when job was created |
| `started` | Timestamp when execution began |
| `finished` | Timestamp when execution completed |

---

## Port Control

### Available Actions

| Subaction | Description | Example |
|-----------|-------------|---------|
| `up` | Administratively enable a port | Set ifAdminStatus to up(1) |
| `down` | Administratively disable a port | Set ifAdminStatus to down(2) |
| `bounce` | Disable then re-enable | down → wait → up |
| `name` | Set port description | Set ifAlias |
| `vlan` | Change access VLAN | Set dot1qPvid |
| `power` | Toggle PoE power | Set pethPsePortAdminEnable |

### Port Control Configuration

```yaml
# Only allow renaming ports (no up/down/vlan)
portctl_nameonly: false

# Allow changing native VLAN
portctl_native_vlan: true

# Exclude wireless APs from port control
portctl_nowaps: false

# Exclude IP phones from port control
portctl_nophones: false

# Allow port control on uplink ports (dangerous!)
portctl_uplinks: false
```

---

## Troubleshooting

### Common Issues

**"Connection refused" when starting web server**
- Check that port 5000 is not in use: `lsof -i :5000`
- Try a different port: `NETDISCO_WEB_PORT=8080 cargo run --bin netdisco-web`

**"Database connection failed"**
- Verify PostgreSQL is running: `pg_isready`
- Check credentials: `psql -U netdisco -h localhost -d netdisco`
- Check environment variables: `env | grep NETDISCO`

**"SNMP timeout" during discovery**
- Verify device is reachable: `ping <device_ip>`
- Test SNMP manually: `snmpget -v2c -c public <device_ip> sysDescr.0`
- Increase timeout: set `snmptimeout: 10000000` (10 seconds)
- Check SNMP community string matches device configuration

**Tests failing**
```bash
# Run tests with verbose output
cargo test --test test_suite -- --nocapture

# Run a specific failing test
cargo test --test test_suite test_name -- --nocapture
```

### Logging

Set the log level via configuration or environment:

```bash
# Via config.yml
log: debug    # options: error, warning, info, debug

# Via environment
RUST_LOG=debug cargo run --bin netdisco-web
RUST_LOG=netdisco=debug,actix_web=info cargo run --bin netdisco-web
```
