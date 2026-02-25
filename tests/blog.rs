use actix_web::{ test, web, App };
use actix_web::http::StatusCode;
use blog::blog::handlers::list_posts;
use blog::config::AppConfig;
use blog::db;
use blog::blog::dto::PostListResponse;
use confik::{ Configuration, EnvSource };
use dotenvy::dotenv;

#[actix_web::test]
async fn test_list_posts_success_flow() {
    dotenv().ok();

    let config = AppConfig::builder().override_with(EnvSource::new()).try_build().unwrap();

    let pool = db::init_pool(&config.pg);

    let app = App::new()
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(pool.clone()))
        .service(list_posts);

    let mut app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/posts").to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::OK, "응답 상태가 200이어야 합니다");

    let body_bytes = test::read_body(resp).await;
    let resp_data: PostListResponse = serde_json
        ::from_slice(&body_bytes)
        .expect("JSON을 PostListResponse로 변환하는 데 실패했습니다");

    assert!(resp_data.total_count > 0, "total_count가 0보다 커야 합니다");
    assert!(!resp_data.posts.is_empty(), "posts가 비어있으면 안 됩니다");
    assert!(resp_data.posts.len() <= 12, "posts 길이가 page_size(12)를 초과하면 안 됩니다");
}
