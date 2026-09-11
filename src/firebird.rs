use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::{IntCounter, IntGauge, IntGaugeVec};
use rsfbclient::prelude::*;
use std::sync::OnceLock;
use tracing::{error, warn};

pub struct FirebirdMetricsCollector {
    connection_string: String,
    descs: OnceLock<Vec<prometheus::core::Desc>>,
}

impl FirebirdMetricsCollector {
    pub fn new(connection_string: String) -> anyhow::Result<Self> {
        Ok(FirebirdMetricsCollector {
            connection_string,
            descs: OnceLock::new(),
        })
    }

    fn gauge(name: &str, description: &str, value: i64) -> anyhow::Result<Box<dyn Collector>> {
        let g = IntGauge::new(name, description)?;
        g.set(value);
        Ok(Box::new(g))
    }

    fn counter(name: &str, description: &str, value: i64) -> anyhow::Result<Box<dyn Collector>> {
        let c = IntCounter::new(name, description)?;
        c.inc_by(value as u64);
        Ok(Box::new(c))
    }

    fn state_metrics<I, F>(
        results: I,
        name: &str,
        description: &str,
        state_fn: F,
    ) -> anyhow::Result<Box<dyn Collector>>
    where
        I: IntoIterator<Item = (i64, i64)>,
        F: Fn(i64) -> String,
    {
        let gauge_vec = IntGaugeVec::new(prometheus::Opts::new(name, description), &["state"])?;
        results
            .into_iter()
            .map(|(state, cnt)| (state_fn(state), cnt))
            .for_each(|(label, cnt)| {
                gauge_vec.with_label_values(&[&label]).set(cnt);
            });
        Ok(Box::new(gauge_vec))
    }

    fn meter(&self) -> anyhow::Result<Vec<Box<dyn Collector>>> {
        let mut metrics: Vec<Box<dyn Collector>> = Vec::new();

        let conn = rsfbclient::builder_pure_rust()
            .from_string(&self.connection_string)?
            .connect()
            .inspect_err(|e| error!("Failed to connect to Firebird: {}", e));

        let up = IntGauge::new("firebird_up", "Firebird database connectivity indicator")?;
        up.set(conn.is_ok() as i64);
        metrics.push(Box::new(up));

        let Ok(mut conn) = conn else {
            return Ok(metrics);
        };

        // Detect server architecture to determine stat group
        let stat_group = if let Ok(Some(server_mode)) = conn.query_first::<_, (String,)>(
            "SELECT RDB$CONFIG_VALUE FROM RDB$CONFIG WHERE RDB$CONFIG_NAME = 'ServerMode'",
            (),
        ) {
            match server_mode.0.as_str() {
                "Super" | "ThreadedDedicated" => 0,
                _ => 1,
            }
        } else {
            warn!("Failed to detect ServerMode, defaulting to aggregated stat group");
            1
        };

        // Database stats
        if let Ok(Some((
            oldest_transaction,
            oldest_active,
            oldest_snapshot,
            next_transaction,
            page_buffers,
            shutdown_mode,
            sweep_interval,
            pages,
            backup_state,
            next_attachment,
            next_statement,
            read_only,
        ))) = conn.query_first::<_, (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64)>(
            "
            SELECT
              MON$OLDEST_TRANSACTION, MON$OLDEST_ACTIVE, MON$OLDEST_SNAPSHOT, MON$NEXT_TRANSACTION,
              MON$PAGE_BUFFERS, MON$SHUTDOWN_MODE, MON$SWEEP_INTERVAL, MON$PAGES, MON$BACKUP_STATE,
              MON$NEXT_ATTACHMENT, MON$NEXT_STATEMENT, MON$READ_ONLY
            FROM MON$DATABASE
            ",
            (),
        ).inspect_err(|e| error!("Failed to query MON$DATABASE: {}", e)) {
            metrics.extend(
                [
                    ("firebird_database_oldest_transaction", "Oldest transaction ID in Firebird database", oldest_transaction),
                    ("firebird_database_oldest_active", "Oldest active transaction ID", oldest_active),
                    ("firebird_database_oldest_snapshot", "Oldest snapshot transaction ID", oldest_snapshot),
                    ("firebird_database_next_transaction", "Next transaction ID", next_transaction),
                    ("firebird_database_page_buffers", "Database page buffers", page_buffers),
                    ("firebird_database_shutdown_mode", "Database shutdown mode", shutdown_mode),
                    ("firebird_database_sweep_interval", "Database sweep interval", sweep_interval),
                    ("firebird_database_pages_total", "Total number of pages in Firebird database", pages),
                    ("firebird_database_backup_state", "Database backup state", backup_state),
                    ("firebird_database_next_attachment", "Next attachment ID", next_attachment),
                    ("firebird_database_next_statement", "Next statement ID", next_statement),
                    ("firebird_database_read_only", "Database read-only flag", read_only),
                ]
                .into_iter()
                .map(|(name, desc, value)| Self::gauge(name, desc, value))
                .collect::<Result<Vec<_>, _>>()?
            );
        }

        // Attachments
        if let Ok(result) = conn
            .query::<_, (i64, i64)>(
                "SELECT MON$STATE, COUNT(*) as cnt FROM MON$ATTACHMENTS GROUP BY MON$STATE",
                (),
            )
            .inspect_err(|e| error!("Failed to query MON$ATTACHMENTS: {}", e))
        {
            metrics.push(Self::state_metrics(
                result,
                "firebird_attachments",
                "Firebird attachments by state",
                |state| match state {
                    0 => "idle".to_string(),
                    1 => "active".to_string(),
                    _ => {
                        warn!("Unknown attachment state: {}", state);
                        state.to_string()
                    }
                },
            )?);
        }

        // Transactions
        if let Ok(result) = conn
            .query::<_, (i64, i64)>(
                "SELECT MON$STATE, COUNT(*) as cnt FROM MON$TRANSACTIONS GROUP BY MON$STATE",
                (),
            )
            .inspect_err(|e| error!("Failed to query MON$TRANSACTIONS: {}", e))
        {
            metrics.push(Self::state_metrics(
                result,
                "firebird_transactions",
                "Firebird transactions by state",
                |state| match state {
                    0 => "idle".to_string(),
                    1 => "active".to_string(),
                    _ => {
                        warn!("Unknown transaction state: {}", state);
                        state.to_string()
                    }
                },
            )?);
        }

        // Statements
        if let Ok(result) = conn
            .query::<_, (i64, i64)>(
                "SELECT MON$STATE, COUNT(*) as cnt FROM MON$STATEMENTS GROUP BY MON$STATE",
                (),
            )
            .inspect_err(|e| error!("Failed to query MON$STATEMENTS: {}", e))
        {
            metrics.push(Self::state_metrics(
                result,
                "firebird_statements",
                "Firebird statements by state",
                |state| match state {
                    0 => "idle".to_string(),
                    1 => "active".to_string(),
                    2 => "stalled".to_string(),
                    _ => {
                        warn!("Unknown statement state: {}", state);
                        state.to_string()
                    }
                },
            )?);
        }

        // IO stats
        if let Ok(Some((reads, writes, fetches, marks))) = conn.query_first::<_, (i64, i64, i64, i64)>(
            "
            SELECT COALESCE(SUM(CAST(MON$PAGE_READS AS INT)), 0), COALESCE(SUM(CAST(MON$PAGE_WRITES AS INT)), 0), COALESCE(SUM(CAST(MON$PAGE_FETCHES AS INT)), 0), COALESCE(SUM(CAST(MON$PAGE_MARKS AS INT)), 0)
            FROM MON$IO_STATS WHERE MON$STAT_GROUP = ?
            ",
            (stat_group,),
        ).inspect_err(|e| error!("Failed to query MON$IO_STATS: {}", e)) {
            metrics.extend(
                [
                    ("firebird_io_page_reads_total", "Total page read operations", reads),
                    ("firebird_io_page_writes_total", "Total page write operations", writes),
                    ("firebird_io_page_fetches_total", "Total page fetch operations", fetches),
                    ("firebird_io_page_marks_total", "Total page mark operations", marks),
                ]
                .into_iter()
                .map(|(name, desc, value)| Self::counter(name, desc, value))
                .collect::<Result<Vec<_>, _>>()?
            );
        }

        // Memory usage
        if let Ok(Some((memory_used, memory_allocated, max_memory_used, max_memory_allocated))) =
            conn.query_first::<_, (i64, i64, i64, i64)>(
                "
                SELECT COALESCE(SUM(CAST(MON$MEMORY_USED AS INT)), 0), COALESCE(SUM(CAST(MON$MEMORY_ALLOCATED AS INT)), 0), COALESCE(SUM(CAST(MON$MAX_MEMORY_USED AS INT)), 0), COALESCE(SUM(CAST(MON$MAX_MEMORY_ALLOCATED AS INT)), 0)
                FROM MON$MEMORY_USAGE WHERE MON$STAT_GROUP = ?
                ",
                (stat_group,),
            ).inspect_err(|e| error!("Failed to query MON$MEMORY_USAGE: {}", e))
        {
            metrics.extend(
                [
                    ("firebird_memory_used_bytes", "Current memory used", memory_used),
                    ("firebird_memory_allocated_bytes", "Current memory allocated", memory_allocated),
                ]
                .into_iter()
                .map(|(name, desc, value)| Self::gauge(name, desc, value))
                .collect::<Result<Vec<_>, _>>()?
            );

            metrics.extend(
                [
                    ("firebird_memory_max_used_bytes", "Maximum memory used", max_memory_used),
                    ("firebird_memory_max_allocated_bytes", "Maximum memory allocated", max_memory_allocated),
                ]
                .into_iter()
                .map(|(name, desc, value)| Self::counter(name, desc, value))
                .collect::<Result<Vec<_>, _>>()?
            );
        }

        Ok(metrics)
    }
}

impl Collector for FirebirdMetricsCollector {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.descs
            .get_or_init(|| {
                let metrics = self.meter().expect("Failed to meter metrics in desc()");
                assert!(
                    metrics.len() > 1,
                    "Expected more than 1 metric, got {}",
                    metrics.len()
                );
                metrics.iter().flat_map(|m| m.desc()).cloned().collect()
            })
            .iter()
            .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut mfs = Vec::new();

        match self.meter() {
            Ok(metrics) => {
                for metric in metrics {
                    mfs.extend(metric.collect());
                }
            }
            Err(e) => {
                error!("Failed to meter metrics: {}", e);
            }
        }

        mfs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tinnitus;

    #[test]
    #[ignore]
    fn test_collector_on_demand() {
        tinnitus().ok();
        let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let collector = FirebirdMetricsCollector::new(connection_string)
            .expect("Failed to create FirebirdMetricsCollector");

        let mfs = collector.collect();
        assert!(!mfs.is_empty());
        assert!(
            mfs.iter().any(|mf| mf.name() == "firebird_up"),
            "Should have firebird_up metric"
        );
    }
}
