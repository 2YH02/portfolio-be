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
async fn project_likes_are_idempotent_and_shared_by_project_id() {
    use actix_web::{test, web, App};
    use blog::project::routes;
    let database = Database::new().await;
    migrations::run(&database.pool).await.unwrap();
    let app = test::init_service(App::new().app_data(web::Data::new(database.pool.clone())).configure(routes::init)).await;
    for (method, expected_count, expected_liked) in [("GET",0,false),("POST",1,true),("POST",1,true),("GET",1,true),("DELETE",0,false),("DELETE",0,false)] {
        let req = test::TestRequest::default().method(method.parse().unwrap()).uri(if method == "GET" {"/projects/7/likes"} else {"/projects/7/like"})
            .insert_header(("Cookie", "portfolio_like_client=12345678-1234-1234-1234-123456789abc")).to_request();
        let response = test::call_service(&app, req).await;
        assert!(response.status().is_success());
        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body["like_count"], expected_count);
        assert_eq!(body["liked"], expected_liked);
    }
    let req = test::TestRequest::post().uri("/projects/7/like").to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 400);
    let req = test::TestRequest::get().uri("/projects/999/likes").to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 404);
    let req = test::TestRequest::post().uri("/projects/7/like").insert_header(("Cookie", "portfolio_like_client=bad")).to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 400);
    let hash = blog::project::service::client_hash("12345678-1234-1234-1234-123456789abc").unwrap();
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let pool = database.pool.clone();
        let hash = hash.clone();
        tasks.push(actix_web::rt::spawn(async move {
            blog::project::service::likes(&pool, 7, &hash, Some(true)).await.unwrap();
        }));
    }
    for task in tasks { task.await.unwrap(); }
    let second_hash = blog::project::service::client_hash("abcdefab-1234-1234-1234-123456789abc").unwrap();
    let second = blog::project::service::likes(&database.pool, 7, &second_hash, Some(true)).await.unwrap();
    assert_eq!(second.like_count, 2);
    assert!(second.liked);
    assert_eq!(blog::project::service::likes(&database.pool, 6, &hash, None).await.unwrap().like_count, 0);
    let removed = blog::project::service::likes(&database.pool, 7, &hash, Some(false)).await.unwrap();
    assert_eq!(removed.like_count, 1);
    assert!(!removed.liked);
    drop(app);
    database.cleanup().await;
}
