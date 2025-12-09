use real_app::proto::user::{UserRequest, UserResponse};
use prost::Message;
use std::io::Cursor;

#[tokio::main]
async fn main() {
    let req = UserRequest { id: "real-123".to_string() };
    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();

    let client = reqwest::Client::new();
    let res = client.post("http://localhost:5150/api/user")
        .header("Content-Type", "application/protobuf")
        .body(buf)
        .send()
        .await
        .unwrap();

    println!("Status: {}", res.status());
    let body = res.bytes().await.unwrap();
    let user_res = UserResponse::decode(Cursor::new(body)).unwrap();
    println!("Response ID: {}", user_res.id);
    println!("Response Name: {}", user_res.name);
    
    assert_eq!(user_res.id, "real-123");
}
