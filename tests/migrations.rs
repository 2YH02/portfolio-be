use blog::{config::AppConfig, db, migrations};
use confik::{Configuration, EnvSource};
use std::sync::atomic::{AtomicU64, Ordering};

static DATABASE_ID: AtomicU64 = AtomicU64::new(0);

// Tests only mutate databases created here, never the configured database.
struct Database {
    admin: db::DbPool,
    pool: db::DbPool,
    name: String,
}

impl Database {
    async fn new() -> Self {
        dotenvy::dotenv().ok();
        let config = AppConfig::builder()
            .override_with(EnvSource::new()).try_build().expect("test database configuration");
        let admin = db::init_pool(&config.pg);
        let name = format!("portfolio_migration_test_{}_{}_{}", std::process::id(),
            chrono::Utc::now().timestamp_micros(), DATABASE_ID.fetch_add(1, Ordering::Relaxed));
        assert!(name.bytes().all(|ch| ch.is_ascii_alphanumeric() || ch == b'_'));
        admin.get().await.unwrap().batch_execute(&format!("CREATE DATABASE {name}"))
            .await.expect("migration tests require a dedicated PostgreSQL role with CREATEDB");
        let mut pg = config.pg.clone();
        pg.url = None;
        pg.dbname = Some(name.clone());
        pg.pool = Some(deadpool_postgres::PoolConfig::new(8));
        Self { admin, pool: db::init_pool(&pg), name }
    }

    async fn cleanup(self) {
        self.pool.close();
        self.admin.get().await.unwrap()
            .batch_execute(&format!("DROP DATABASE {} WITH (FORCE)", self.name)).await.unwrap();
    }
}

#[actix_web::test]
async fn fresh_database_and_repeat_preserve_history_and_content() {
    let database = Database::new().await;
    migrations::run(&database.pool).await.unwrap();
    let client = database.pool.get().await.unwrap();
    let first = client.query_one(
        "SELECT checksum, applied_at FROM public.portfolio_schema_migrations WHERE version = 1", &[]
    ).await.unwrap();
    client.execute(
        "INSERT INTO guestbook_entries (scope, nickname, message, client_hash)
         VALUES ('test', 'visitor', 'keep this content', repeat('a', 64))", &[]
    ).await.unwrap();
    migrations::run(&database.pool).await.unwrap();
    let second = client.query_one(
        "SELECT checksum, applied_at FROM public.portfolio_schema_migrations WHERE version = 1", &[]
    ).await.unwrap();
    assert_eq!(first.get::<_, String>(0), second.get::<_, String>(0));
    assert_eq!(first.get::<_, chrono::DateTime<chrono::Utc>>(1),
        second.get::<_, chrono::DateTime<chrono::Utc>>(1));
    assert_eq!(client.query_one("SELECT message FROM guestbook_entries", &[])
        .await.unwrap().get::<_, String>(0), "keep this content");
    drop(client);
    database.cleanup().await;
}

#[actix_web::test]
async fn failed_migration_rolls_back_ddl_and_history_then_can_retry() {
    let database = Database::new().await;
    let client = database.pool.get().await.unwrap();
    client.batch_execute(
        "CREATE TABLE guestbook_rate_limits (client_hash CHAR(64) PRIMARY KEY)"
    ).await.unwrap();
    assert!(migrations::run(&database.pool).await.is_err());
    for table in ["public.guestbook_entries", "public.portfolio_schema_migrations"] {
        assert!(client.query_one("SELECT to_regclass($1)::text", &[&table])
            .await.unwrap().get::<_, Option<String>>(0).is_none());
    }
    client.batch_execute("ALTER TABLE guestbook_rate_limits ADD COLUMN last_submitted_at TIMESTAMP NOT NULL DEFAULT NOW()")
        .await.unwrap();
    migrations::run(&database.pool).await.unwrap();
    drop(client);
    database.cleanup().await;
}

#[actix_web::test]
async fn concurrent_runners_apply_one_history_entry() {
    let database = Database::new().await;
    let first_pool = database.pool.clone();
    let second_pool = database.pool.clone();
    let first = actix_web::rt::spawn(async move { migrations::run(&first_pool).await });
    let second = actix_web::rt::spawn(async move { migrations::run(&second_pool).await });
    first.await.unwrap().unwrap();
    second.await.unwrap().unwrap();
    let client = database.pool.get().await.unwrap();
    assert_eq!(client.query_one("SELECT count(*) FROM public.portfolio_schema_migrations", &[])
        .await.unwrap().get::<_, i64>(0), 2);
    drop(client);
    database.cleanup().await;
}

#[actix_web::test]
async fn newer_additive_history_allows_rollback_but_changed_known_history_fails() {
    let database = Database::new().await;
    migrations::run(&database.pool).await.unwrap();
    let client = database.pool.get().await.unwrap();
    client.batch_execute(
        "ALTER TABLE guestbook_entries ADD COLUMN future_note TEXT;
         INSERT INTO public.portfolio_schema_migrations (version, name, checksum)
         VALUES (3, 'future_additive', 'future-checksum')"
    ).await.unwrap();
    migrations::run(&database.pool).await.unwrap();
    client.batch_execute("UPDATE public.portfolio_schema_migrations SET checksum = 'changed' WHERE version = 1")
        .await.unwrap();
    assert!(migrations::run(&database.pool).await.is_err());
    drop(client);
    database.cleanup().await;
}

#[actix_web::test]
async fn newer_history_cannot_hide_missing_known_version() {
    let database = Database::new().await;
    migrations::run(&database.pool).await.unwrap();
    let client = database.pool.get().await.unwrap();
    client.batch_execute(
        "UPDATE public.portfolio_schema_migrations SET version = 3 WHERE version = 1"
    ).await.unwrap();
    assert!(migrations::run(&database.pool).await.is_err());
    drop(client);
    database.cleanup().await;
}

#[actix_web::test]
async fn lock_wait_times_out_and_releases_connection_settings() {
    let database = Database::new().await;
    let mut client = database.pool.get().await.unwrap();
    let transaction = client.transaction().await.unwrap();
    transaction.query_one("SELECT pg_advisory_xact_lock(7281946201::bigint)", &[]).await.unwrap();
    let started = std::time::Instant::now();
    let error = migrations::run(&database.pool).await.unwrap_err();
    let pg_error = error.downcast_ref::<tokio_postgres::Error>().expect("PostgreSQL timeout");
    assert_eq!(pg_error.code(), Some(&tokio_postgres::error::SqlState::LOCK_NOT_AVAILABLE));
    assert!(started.elapsed() < std::time::Duration::from_secs(15));
    transaction.rollback().await.unwrap();
    drop(client);
    migrations::run(&database.pool).await.unwrap();
    let client = database.pool.get().await.unwrap();
    assert_eq!(client.query_one("SHOW lock_timeout", &[]).await.unwrap().get::<_, String>(0), "0");
    assert_eq!(client.query_one("SHOW statement_timeout", &[]).await.unwrap().get::<_, String>(0), "0");
    drop(client);
    database.cleanup().await;
}
