use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use proof_audit::{AuditEntry, FcaAuditPack, build_fca_pack};
use proof_verify::Divergence;
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::str::FromStr;

use crate::state::RecentEvent;

#[derive(Clone)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    pub async fn open(path: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(opts)
            .await
            .with_context(|| format!("failed to open database at {}", path))?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("database migration failed")?;
        Ok(())
    }

    // ─── audit ───────────────────────────────────────────────────────────────

    pub async fn insert_audit(&self, entry: &AuditEntry) -> Result<()> {
        let id   = entry.id.to_string();
        let ts   = entry.timestamp.to_rfc3339();
        let data = serde_json::to_string(entry)?;

        sqlx::query(
            "INSERT OR IGNORE INTO audit_entries (id, timestamp, spec_name, data) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&ts)
        .bind(&entry.spec_name)
        .bind(&data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_audit(&self, spec_name: Option<&str>, limit: i64) -> Result<Vec<AuditEntry>> {
        let rows = match spec_name {
            Some(name) => {
                sqlx::query(
                    "SELECT data FROM audit_entries WHERE spec_name = ? ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(name)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT data FROM audit_entries ORDER BY timestamp DESC LIMIT ?",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        rows.iter()
            .map(|row| {
                let data: String = row.try_get("data")?;
                serde_json::from_str::<AuditEntry>(&data)
                    .context("failed to deserialise audit entry")
            })
            .collect()
    }

    pub async fn get_all_audit(&self) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query("SELECT data FROM audit_entries ORDER BY timestamp ASC")
            .fetch_all(&self.pool)
            .await?;

        rows.iter()
            .map(|row| {
                let data: String = row.try_get("data")?;
                serde_json::from_str::<AuditEntry>(&data).context("deserialise")
            })
            .collect()
    }

    pub async fn fca_pack(&self, spec_name: &str) -> Result<FcaAuditPack> {
        let all = self.get_all_audit().await?;
        Ok(build_fca_pack(spec_name, &all))
    }

    // ─── divergences ─────────────────────────────────────────────────────────

    pub async fn insert_divergence(&self, d: &Divergence) -> Result<()> {
        let id   = d.id.to_string();
        let ts   = d.detected_at.to_rfc3339();
        let data = serde_json::to_string(d)?;

        sqlx::query(
            "INSERT OR IGNORE INTO divergences (id, detected_at, spec_name, customer_id, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&ts)
        .bind(&d.spec_name)
        .bind(&d.customer_id)
        .bind(&data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_divergences(&self, spec_name: Option<&str>) -> Result<Vec<Divergence>> {
        let rows = match spec_name {
            Some(name) => {
                sqlx::query(
                    "SELECT data FROM divergences WHERE spec_name = ? AND resolved = 0 ORDER BY detected_at DESC",
                )
                .bind(name)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT data FROM divergences WHERE resolved = 0 ORDER BY detected_at DESC",
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        rows.iter()
            .map(|row| {
                let data: String = row.try_get("data")?;
                serde_json::from_str::<Divergence>(&data).context("deserialise divergence")
            })
            .collect()
    }

    pub async fn resolve_divergence(&self, id: &str) -> Result<bool> {
        let result =
            sqlx::query("UPDATE divergences SET resolved = 1 WHERE id = ? AND resolved = 0")
                .bind(id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_divergences(&self, spec_name: Option<&str>) -> Result<i64> {
        let row = match spec_name {
            Some(name) => sqlx::query(
                "SELECT COUNT(*) as c FROM divergences WHERE spec_name = ? AND resolved = 0",
            )
            .bind(name)
            .fetch_one(&self.pool)
            .await?,
            None => sqlx::query(
                "SELECT COUNT(*) as c FROM divergences WHERE resolved = 0",
            )
            .fetch_one(&self.pool)
            .await?,
        };
        Ok(row.try_get("c")?)
    }

    // ─── events ──────────────────────────────────────────────────────────────

    pub async fn insert_event(&self, event: &RecentEvent) -> Result<()> {
        let ts = event.timestamp.to_rfc3339();
        let ok = if event.ok { 1i64 } else { 0i64 };

        sqlx::query(
            "INSERT INTO verification_events (customer_id, spec_name, event_type, ok, timestamp) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&event.customer_id)
        .bind(&event.spec_name)
        .bind(&event.event_type)
        .bind(ok)
        .bind(&ts)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_events(&self, limit: i64) -> Result<Vec<RecentEvent>> {
        let rows = sqlx::query(
            "SELECT customer_id, spec_name, event_type, ok, timestamp FROM verification_events ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(|row| {
                let ts_str: String = row.try_get("timestamp")?;
                let ok: i64        = row.try_get("ok")?;
                let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(RecentEvent {
                    customer_id: row.try_get("customer_id")?,
                    spec_name:   row.try_get("spec_name")?,
                    event_type:  row.try_get("event_type")?,
                    ok:          ok != 0,
                    timestamp,
                })
            })
            .collect()
    }

    pub async fn count_events(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as c FROM verification_events")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get("c")?)
    }
}