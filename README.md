# Firebird Exporter

[GitHub](https://github.com/hacker/firebird_exporter) • Prometheus exporter for Firebird databases.

Only tested with Firebird 5. Just the metrics I came up with in times of need.

## Quick Start

### Docker

```bash
docker run -e DATABASE_URL="firebird://host.docker.internal:3050/path/to/database.fdb" \
  -p 9123:9123 \
  ghcr.io/hacker/firebird_exporter:latest
```

Metrics available at `http://localhost:9123/metrics`

### Kubernetes

Helm chart available at `oci://ghcr.io/hacker/firebird_exporter-chart/firebird-exporter`

## Configuration

Environment variables:

- `DATABASE_URL` - Firebird connection string
- `HTTP_LISTEN_ADDR` - Listen address (default: `0.0.0.0:9123`)

When built with the `dotenv` feature (default), load from `.env` file.

## Running

```bash
./firebird_exporter
```

Metrics available at `http://localhost:9123/metrics` (or whatever you're listening to)

Health check: `./firebird_exporter healthcheck`

## Grafana Dashboard

A pre-built dashboard is available in the
[releases](https://github.com/hacker/firebird_exporter/releases) as
`firebird-exporter-dashboard-<version>.json`.

## Metrics

### Connectivity

- `firebird_up` - Database connectivity (0 or 1)

### Database

- `firebird_database_oldest_transaction` - Oldest transaction ID
- `firebird_database_oldest_active` - Oldest active transaction ID
- `firebird_database_oldest_snapshot` - Oldest snapshot transaction ID
- `firebird_database_next_transaction` - Next transaction ID
- `firebird_database_page_buffers` - Database page buffers
- `firebird_database_shutdown_mode` - Shutdown mode
- `firebird_database_sweep_interval` - Sweep interval
- `firebird_database_pages_total` - Total pages
- `firebird_database_backup_state` - Backup state
- `firebird_database_next_attachment` - Next attachment ID
- `firebird_database_next_statement` - Next statement ID
- `firebird_database_read_only` - Read-only flag (0 or 1)

### Attachments, Transactions, Statements

- `firebird_attachments` - Attachments by state (`idle`, `active`)
- `firebird_transactions` - Transactions by state (`idle`, `active`)
- `firebird_statements` - Statements by state (`idle`, `active`, `stalled`)

### I/O

- `firebird_io_page_reads_total` - Page read operations
- `firebird_io_page_writes_total` - Page write operations
- `firebird_io_page_fetches_total` - Page fetch operations
- `firebird_io_page_marks_total` - Page mark operations

### Memory

- `firebird_memory_used_bytes` - Current memory used
- `firebird_memory_allocated_bytes` - Current memory allocated
- `firebird_memory_max_used_bytes` - Maximum memory used
- `firebird_memory_max_allocated_bytes` - Maximum memory allocated

## Building

```bash
cargo build --release
```

The `dotenv` feature (enabled by default) allows loading environment variables
from a `.env` file. To build without it:

```bash
cargo build --release --no-default-features
```
