use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use demo_app::{app, loco_protobuf::Protobuf, user::{UserRequest, UserResponse}};
use prost::Message;
use tower::ServiceExt; 

#[tokio::test]
async fn test_protobuf_request_response() {
    let app = app();

    let user_req = UserRequest { id: "123".to_string() };
    let mut buf = Vec::new();
    user_req.encode(&mut buf).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user")
                .header(http::header::CONTENT_TYPE, "application/protobuf")
                .body(Body::from(buf))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
        "application/protobuf"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user_res = UserResponse::decode(body).unwrap();

    assert_eq!(user_res.id, "123");
    assert_eq!(user_res.name, "John Doe");
}
