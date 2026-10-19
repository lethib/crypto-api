// These tests cover the HTTP boundary (routing, extraction, status codes, state wiring).
// The signing logic itself is covered by the unit tests in src/signing.
mod common;

use serde_json::{Value, json};
use tower::ServiceExt;

use common::{ResponseBodyToJson, app, post_json};

fn payload() -> Value {
    json!({
        "message": "Hello World",
        "timestamp": 1616161616
    })
}

async fn sign(payload: &Value) -> String {
    let request = post_json("/sign", &payload.to_string());
    let body = app().oneshot(request).await.unwrap().body_to_json().await;

    body["signature"].as_str().unwrap().to_owned()
}

mod verify {
    use axum::http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    use super::common::{app, post_json};
    use super::{payload, sign};

    #[tokio::test]
    async fn it_should_accept_a_signature_produced_by_sign_with_204() {
        let signature = sign(&payload()).await;
        let body = json!({ "signature": signature, "data": payload() });
        let request = post_json("/verify", &body.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn it_should_accept_the_data_whatever_the_order_of_its_properties() {
        let signature = sign(&payload()).await;
        let reordered = r#"{"timestamp":1616161616,"message":"Hello World"}"#;
        let body = format!(r#"{{"signature":"{signature}","data":{reordered}}}"#);
        let request = post_json("/verify", &body);
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn it_should_reject_tampered_data_with_400() {
        let signature = sign(&payload()).await;
        let tampered = json!({ "message": "Goodbye World", "timestamp": 1616161616 });
        let body = json!({ "signature": signature, "data": tampered });
        let request = post_json("/verify", &body.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn it_should_reject_a_tampered_signature_with_400() {
        let mut signature = sign(&payload()).await;
        let last = signature.pop().unwrap();
        signature.push(if last == '0' { '1' } else { '0' });
        let body = json!({ "signature": signature, "data": payload() });
        let request = post_json("/verify", &body.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn it_should_reject_a_non_hex_signature_with_400() {
        let body = json!({ "signature": "a1b2c3d4e5f6g7h8i9j0", "data": payload() });
        let request = post_json("/verify", &body.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

mod invalid_requests {
    use axum::http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    use super::common::{app, post_json};

    #[tokio::test]
    async fn it_should_reject_a_verify_payload_without_data_with_422() {
        let payload = json!({ "signature": "a1b2c3" });
        let request = post_json("/verify", &payload.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn it_should_reject_a_verify_payload_without_signature_with_422() {
        let payload = json!({ "data": { "message": "Hello World" } });
        let request = post_json("/verify", &payload.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn it_should_reject_a_non_string_signature_with_422() {
        let payload = json!({ "signature": 42, "data": { "message": "Hello World" } });
        let request = post_json("/verify", &payload.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
