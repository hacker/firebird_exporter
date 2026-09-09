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
