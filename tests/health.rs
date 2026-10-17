use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use crypto_api::AppState;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

trait ResponseBodyToJson {
    async fn body_to_json(self) -> serde_json::Value;
}

impl ResponseBodyToJson for Response<Body> {
    async fn body_to_json(self) -> serde_json::Value {
        let body = self.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }
}

#[tokio::test]
async fn health_returns_ok() {
    let request = Request::get("/health").body(Body::empty()).unwrap();
    let response = crypto_api::app(AppState::with_base64_encryptor())
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body_to_json().await, json!({ "status":"ok" }))
}
