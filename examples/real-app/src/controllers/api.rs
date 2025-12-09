use loco_rs::prelude::*;
use loco_protobuf::Protobuf;
use crate::proto::user::{UserRequest, UserResponse};

async fn create_user(
    Protobuf(req): Protobuf<UserRequest>,
) -> Result<Protobuf<UserResponse>> {
    println!("Received user request: {:?}", req);
    Ok(Protobuf(UserResponse {
        id: req.id,
        name: "Real App User".to_string(),
        email: "real@app.com".to_string(),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/user", post(create_user))
}
