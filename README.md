# Firebird Exporter

Prometheus exporter for Firebird databases.

Only tested with Firebird 5. Just the metrics I came up with in times of need.

## Building

```bash
cargo build --release
```

## Configuration

Set environment variables:

- `DATABASE_URL` - Firebird database connection URL (required)
- `HTTP_LISTEN_ADDR` - Listen address (default: `0.0.0.0:9123`)

Example:

```bash
export DATABASE_URL="firebird://localhost:3050/path/to/database.fdb"
export HTTP_LISTEN_ADDR="127.0.0.1:9123"
```

## Running

```bash
./firebird_exporter
```

Metrics available at `http://localhost:9123/metrics` (or whatever you're listening to)

Health check: `./firebird_exporter healthcheck`

## Metrics

### Connectivity

- `firebird_up` - Firebird database connectivity indicator (0 or 1)

### Database Statistics

- `firebird_database_oldest_transaction` - Oldest transaction ID in Firebird database
- `firebird_database_oldest_active` - Oldest active transaction ID
- `firebird_database_oldest_snapshot` - Oldest snapshot transaction ID
- `firebird_database_next_transaction` - Next transaction ID
- `firebird_database_page_buffers` - Database page buffers
- `firebird_database_shutdown_mode` - Database shutdown mode
- `firebird_database_sweep_interval` - Database sweep interval
- `firebird_database_pages_total` - Total number of pages in Firebird database
- `firebird_database_backup_state` - Database backup state
- `firebird_database_next_attachment` - Next attachment ID
- `firebird_database_next_statement` - Next statement ID
- `firebird_database_read_only` - Database read-only flag (0 or 1)

### Attachments

- `firebird_attachments` - Firebird attachments by state (labels: `state` with values `idle`, `active`)

### Transactions

- `firebird_transactions` - Firebird transactions by state (labels: `state` with values `idle`, `active`)

### Statements

- `firebird_statements` - Firebird statements by state (labels: `state` with values `idle`, `active`, `stalled`)

### I/O Statistics

- `firebird_io_page_reads_total` - Total page read operations (counter)
- `firebird_io_page_writes_total` - Total page write operations (counter)
- `firebird_io_page_fetches_total` - Total page fetch operations (counter)
- `firebird_io_page_marks_total` - Total page mark operations (counter)

### Memory Usage

- `firebird_memory_used_bytes` - Current memory used (gauge)
- `firebird_memory_allocated_bytes` - Current memory allocated (gauge)
- `firebird_memory_max_used_bytes` - Maximum memory used (counter)
- `firebird_memory_max_allocated_bytes` - Maximum memory allocated (counter)
