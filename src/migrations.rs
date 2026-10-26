use sha2::{Digest, Sha256};

use crate::db::DbPool;

// Append new versions; never edit SQL that has already been deployed.
const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "create_guestbook",
    include_str!("../sql/guestbook_schema.sql"),
)];

pub async fn run(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    if MIGRATIONS.iter().any(|migration| migration.0 <= 0)
        || MIGRATIONS.windows(2).any(|pair| pair[0].0 >= pair[1].0)
    {
        return Err("migration versions must be positive and strictly increasing".into());
    }
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    transaction
        .batch_execute(
            "SET LOCAL lock_timeout = '5s';
             SET LOCAL statement_timeout = '60s';
             SET LOCAL search_path = public, pg_catalog",
        )
        .await?;
    // Transaction-scoped lock serializes concurrent deployments and releases on failure.
    transaction
        .query_one("SELECT pg_advisory_xact_lock(7281946201::bigint)", &[])
        .await?;
    transaction
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS public.portfolio_schema_migrations (
            version BIGINT PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        )
        .await?;

    let applied = transaction.query(
        "SELECT version, name, checksum FROM public.portfolio_schema_migrations ORDER BY version",
        &[],
    ).await?;
    for row in &applied {
        let version: i64 = row.get("version");
        let Some((_, name, sql)) = MIGRATIONS.iter().find(|migration| migration.0 == version)
        else {
            let latest = MIGRATIONS.last().map_or(0, |migration| migration.0);
            if version <= latest {
                return Err(format!("unknown historical database migration {version}").into());
            }
            // Rollback is supported only for additive, backward-compatible migrations.
            tracing::warn!("database migration {version} is newer than this application; preserving it");
            continue;
        };
        let checksum = checksum(sql);
        if row.get::<_, String>("checksum") != checksum || row.get::<_, String>("name") != *name {
            return Err(format!(
                "migration {version} changed after being applied; add a new version instead"
            )
            .into());
        }
    }

    // Never backfill an earlier migration after a later one has already run.
    for migration in MIGRATIONS {
        if !applied.iter().any(|row| row.get::<_, i64>("version") == migration.0)
            && applied.iter().any(|row| row.get::<_, i64>("version") > migration.0)
        {
            return Err("database has incomplete migration history".into());
        }
    }

    for &(version, name, sql) in MIGRATIONS {
        if applied
            .iter()
            .any(|row| row.get::<_, i64>("version") == version)
        {
            continue;
        }
        tracing::info!("applying database migration {version}: {name}");
        transaction.batch_execute(sql).await?;
        let checksum = checksum(sql);
        transaction.execute(
            "INSERT INTO public.portfolio_schema_migrations (version, name, checksum) VALUES ($1, $2, $3)",
            &[&version, &name, &checksum],
        ).await?;
    }
    validate_guestbook(&transaction).await?;
    transaction.commit().await?;
    Ok(())
}

async fn validate_guestbook(
    transaction: &tokio_postgres::Transaction<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    // IF NOT EXISTS must not silently adopt incompatible pre-existing tables.
    for (table, expected) in [
        ("guestbook_entries", &[
            ("id", "integer", true), ("scope", "character varying(64)", true),
            ("nickname", "character varying(16)", true),
            ("message", "character varying(120)", true),
            ("status", "character varying(16)", true),
            ("moderation_reason", "character varying(64)", false),
            ("client_hash", "character(64)", true),
            ("created_at", "timestamp without time zone", true),
        ][..]),
        ("guestbook_rate_limits", &[
            ("client_hash", "character(64)", true),
            ("last_submitted_at", "timestamp without time zone", true),
        ][..]),
    ] {
        let rows = transaction.query(
            "SELECT a.attname::text, format_type(a.atttypid, a.atttypmod), a.attnotnull
             FROM pg_catalog.pg_attribute a
             JOIN pg_catalog.pg_class c ON c.oid = a.attrelid
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = 'public' AND c.relname = $1
               AND c.relkind IN ('r', 'p') AND a.attnum > 0 AND NOT a.attisdropped",
            &[&table],
        ).await?;
        for &(column, data_type, not_null) in expected {
            if !rows.iter().any(|row| {
                row.get::<_, String>(0) == column
                    && row.get::<_, String>(1) == data_type
                    && row.get::<_, bool>(2) == not_null
            }) {
                return Err(format!("incompatible guestbook schema: {table}.{column}").into());
            }
        }
    }
    Ok(())
}

fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.replace("\r\n", "\n").as_bytes()))
}
