use axum::{routing::post, Router};
pub use loco_protobuf;
use loco_protobuf::Protobuf;

// Include generated proto modules
pub mod user {
    include!(concat!(env!("OUT_DIR"), "/user.rs"));
}
use user::{UserRequest, UserResponse};

async fn create_user(
    Protobuf(req): Protobuf<UserRequest>,
) -> Protobuf<UserResponse> {
    println!("Received user request: {:?}", req);
    Protobuf(UserResponse {
        id: req.id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

pub fn app() -> Router {
    Router::new().route("/user", post(create_user))
}
