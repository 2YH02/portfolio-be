use actix_web::{App, http::StatusCode, test, web};
use blog::{config::AppConfig, db, guestbook, migrations};
use confik::{Configuration, EnvSource};

#[actix_web::test]
async fn writing_persists_pending_entry_and_enforces_then_expires_cooldown() {
    dotenvy::dotenv().ok();
    let config = AppConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .unwrap();
    let admin = db::init_pool(&config.pg);
    let name = format!(
        "portfolio_guestbook_test_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    );
    assert!(
        name.bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
    );
    admin
        .get()
        .await
        .unwrap()
        .batch_execute(&format!("CREATE DATABASE {name}"))
        .await
        .unwrap();
    let mut pg = config.pg.clone();
    pg.url = None;
    pg.dbname = Some(name.clone());
    let pool = db::init_pool(&pg);
    migrations::run(&pool).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(
                guestbook::service::ModerationRules::default(),
            ))
            .service(guestbook::handlers::create)
            .service(guestbook::handlers::list),
    )
    .await;
    let submit = || {
        test::TestRequest::post()
            .uri("/guestbook/castmorrow")
            .insert_header(("x-guestbook-token", "integration-test-visitor"))
            .set_json(serde_json::json!({"nickname":"visitor", "message":"hello sea"}))
            .to_request()
    };

    let response = test::call_service(&app, submit()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(body["visible"], false);
    let client = pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT message, status, moderation_reason FROM guestbook_entries",
            &[],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>("message"), "hello sea");
    assert_eq!(row.get::<_, String>("status"), "pending");
    assert_eq!(
        row.get::<_, String>("moderation_reason"),
        "rules_unavailable"
    );

    let response = test::call_service(&app, submit()).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert!(body["error"].as_str().unwrap().contains("요청"));
    client
        .batch_execute(
            "UPDATE guestbook_rate_limits SET last_submitted_at = NOW() - INTERVAL '31 seconds'",
        )
        .await
        .unwrap();
    let response = test::call_service(&app, submit()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        client
            .query_one("SELECT count(*) FROM guestbook_entries", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        2
    );

    let request = test::TestRequest::get()
        .uri("/guestbook/castmorrow")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, request).await;
    assert_eq!(body["entries"], serde_json::json!([]));
    drop(client);
    drop(app);
    pool.close();
    admin
        .get()
        .await
        .unwrap()
        .batch_execute(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .await
        .unwrap();
}
