use firebird_exporter::{FirebirdMetricsCollector, tinnitus};
use anyhow::Context;
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tiny_http::{Response, Server};
use tracing::{error, info};

fn main() -> anyhow::Result<()> {
    tinnitus()?;

    let listen_addr =
        std::env::var("HTTP_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:9123".to_string());

    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        match ureq::get(&format!("http://{}/healthz", listen_addr)).call() {
            Ok(resp) if resp.status() == 200 => std::process::exit(0),
            Ok(resp) => error!("Healthcheck failed: HTTP {}", resp.status()),
            Err(e) => error!("Healthcheck request failed: {}", e),
        }
        std::process::exit(1);
    }

    info!("Starting Firebird Prometheus exporter");

    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL is not set")?;

    info!("Listening on {}", listen_addr);

    let collector = FirebirdMetricsCollector::new(database_url)?;

    let registry = Registry::new();
    registry.register(Box::new(collector))?;

    let registry = Arc::new(registry);
    let server = Server::http(&listen_addr)
        .map_err(|e| anyhow::anyhow!("Failed to start HTTP server: {}", e))?;

    for request in server.incoming_requests() {
        let response = match request.url() {
            "/metrics" => {
                let encoder = TextEncoder::new();
                let metric_families = registry.gather();
                let mut buffer = vec![];
                match encoder.encode(&metric_families, &mut buffer) {
                    Ok(_) => Response::from_string(String::from_utf8_lossy(&buffer).into_owned())
                        .with_header(
                            tiny_http::Header::from_bytes(
                                b"Content-Type",
                                b"text/plain; version=0.0.4",
                            )
                            .unwrap(),
                        ),
                    Err(e) => {
                        tracing::error!("Failed to encode metrics: {}", e);
                        Response::from_string("Error encoding metrics").with_status_code(500)
                    }
                }
            }
            "/healthz" => Response::from_string("OK\n"),
            _ => Response::from_string("Not found").with_status_code(404),
        };
        let _ = request.respond(response);
    }

    info!("Exporter shut down");
    Ok(())
}
