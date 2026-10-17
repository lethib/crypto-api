// These tests cover the HTTP boundary (routing, extraction, status codes, state wiring).
// The encryption logic itself is covered by the unit tests in src/encryption.
mod common;

use serde_json::{Value, json};

fn payload() -> Value {
    json!({
        "name": "John Doe",
        "age": 30,
        "contact": {
            "email": "john@example.com",
            "phone": "123-456-7890"
        }
    })
}

mod encrypt {
    use axum::http::StatusCode;
    use tower::ServiceExt;

    use super::common::{ResponseBodyToJson, app, post_json};
    use super::payload;

    #[tokio::test]
    async fn it_should_encrypt_every_depth_one_property() {
        let request = post_json("/encrypt", &payload().to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.body_to_json().await;
        let encrypted = body.as_object().unwrap();
        let original = payload();
        let original = original.as_object().unwrap();

        assert!(encrypted.keys().eq(original.keys()));
        for value in encrypted.values() {
            assert!(value.as_str().unwrap().starts_with("enc:"));
        }
    }
}

mod decrypt {
    use axum::http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    use super::common::{ResponseBodyToJson, app, post_json};
    use super::payload;

    #[tokio::test]
    async fn it_should_return_the_original_payload_from_encrypt_output() {
        let request = post_json("/encrypt", &payload().to_string());
        let encrypted = app().oneshot(request).await.unwrap().body_to_json().await;

        let request = post_json("/decrypt", &encrypted.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body_to_json().await, payload());
    }

    #[tokio::test]
    async fn it_should_leave_unencrypted_properties_unchanged() {
        let payload = json!({
            "name": "enc:IkpvaG4gRG9lIg==",
            "age": "enc:MzA=",
            "birth_date": "1998-11-19"
        });
        let request = post_json("/decrypt", &payload.to_string());
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.body_to_json().await,
            json!({
                "name": "John Doe",
                "age": 30,
                "birth_date": "1998-11-19"
            })
        );
    }
}

mod invalid_requests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::common::{app, post_json};

    #[tokio::test]
    async fn it_should_reject_malformed_json_with_400() {
        let request = post_json("/encrypt", r#"{"name":"#);
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn it_should_reject_a_non_object_payload_with_422() {
        let request = post_json("/encrypt", "[1, 2]");
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn it_should_reject_a_missing_content_type_with_415() {
        let request = Request::post("/encrypt")
            .body(Body::from(r#"{"name":"John Doe"}"#))
            .unwrap();
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn it_should_reject_get_with_405() {
        let request = Request::get("/encrypt").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
