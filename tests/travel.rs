use actix_web::cookie::Cookie;
use actix_web::http::StatusCode;
use actix_web::{App, test, web};
use blog::config::AppConfig;
use blog::db;
use blog::travel::routes;
use confik::{Configuration, EnvSource};
use dotenvy::dotenv;
use serde_json::{Value, json};

const TEST_SLUG: &str = "codex-travel-flow";

#[actix_web::test]
async fn test_travel_like_comment_reaction_flow() {
    dotenv().ok();

    let config = AppConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .unwrap();
    let pool = db::init_pool(&config.pg);
    setup_db(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::init),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/travel/posts/{TEST_SLUG}/likes"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["like_count"], 0);

    let req = test::TestRequest::post()
        .uri(&format!("/travel/posts/{TEST_SLUG}/like"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let like_cookie = response_cookie(&resp, "travel_liked_posts");
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["like_count"], 1);

    let req = test::TestRequest::post()
        .uri(&format!("/travel/posts/{TEST_SLUG}/like"))
        .cookie(like_cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let req = test::TestRequest::delete()
        .uri(&format!("/travel/posts/{TEST_SLUG}/like"))
        .cookie(like_cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["like_count"], 0);

    let req = test::TestRequest::post()
        .uri(&format!("/travel/posts/{TEST_SLUG}/comments"))
        .set_json(json!({
            "author": "여행자",
            "password": "1234",
            "content": "좋은 글이네요!"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = test::read_body_json(resp).await;
    let comment_id = body["id"].as_i64().unwrap() as i32;

    let req = test::TestRequest::get()
        .uri(&format!("/travel/posts/{TEST_SLUG}/comments"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["comments"].as_array().unwrap().len(), 1);
    assert_eq!(
        body["comments"][0]["reactions"].as_array().unwrap().len(),
        0
    );

    let req = test::TestRequest::post()
        .uri(&format!("/travel/comments/{comment_id}/react"))
        .set_json(json!({ "emoji": "👍" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let reaction_cookie = response_cookie(&resp, &format!("travel_reactions_{comment_id}"));
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["emoji"], "👍");
    assert_eq!(body["count"], 1);

    let req = test::TestRequest::get()
        .uri(&format!("/travel/posts/{TEST_SLUG}/comments"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["comments"][0]["reactions"][0]["emoji"], "👍");
    assert_eq!(body["comments"][0]["reactions"][0]["count"], 1);

    let req = test::TestRequest::post()
        .uri(&format!("/travel/comments/{comment_id}/react"))
        .cookie(reaction_cookie)
        .set_json(json!({ "emoji": "👍" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let req = test::TestRequest::delete()
        .uri(&format!("/travel/comments/{comment_id}"))
        .set_json(json!({ "password": "wrong" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let req = test::TestRequest::delete()
        .uri(&format!("/travel/comments/{comment_id}"))
        .set_json(json!({ "password": "1234" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

async fn setup_db(pool: &db::DbPool) {
    let client = pool.get().await.unwrap();
    client
        .batch_execute(include_str!("../sql/travel_schema.sql"))
        .await
        .unwrap();
    client
        .execute("DELETE FROM travel_comments WHERE slug = $1", &[&TEST_SLUG])
        .await
        .unwrap();
    client
        .execute("DELETE FROM travel_likes WHERE slug = $1", &[&TEST_SLUG])
        .await
        .unwrap();
}

fn response_cookie(resp: &actix_web::dev::ServiceResponse, name: &str) -> Cookie<'static> {
    resp.response()
        .cookies()
        .find(|cookie| cookie.name() == name)
        .expect("response should set cookie")
        .into_owned()
}
