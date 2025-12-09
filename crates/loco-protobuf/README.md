# Loco ProtoBuf

A helper crate to easily integrate Protocol Buffers with [Loco.rs](https://loco.rs), offering Axum-compatible extractors and responses.

## Features

- **`Protobuf<T>` Extractor**: Automatically decodes `application/protobuf` request bodies into your Prost-generated structs.
- **`Protobuf<T>` Response**: Automatically encodes your structs into Protobuf format and sets the `Content-Type: application/protobuf` header.
- **Axum 0.8 / Loco 0.16 Support**: Built for the latest Loco versions.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
loco-protobuf = "0.1"
prost = "0.13"
axum = "0.8"

[build-dependencies]
prost-build = "0.13"
```

## Setup

1. **Define your Protos**: Create a `proto` directory and add your `.proto` files (e.g., `proto/user.proto`).

2. **Configure Build Script**: Create or update `build.rs`:

   ```rust
   fn main() {
       prost_build::compile_protos(&["proto/user.proto"], &["proto"]).unwrap();
   }
   ```

   *Tip*: Use `protoc-bin-vendored` if you don't want to install `protoc` manually.

3. **Include Generated Code**: In your `src/lib.rs` or `src/proto.rs`:

   ```rust
   pub mod user {
       include!(concat!(env!("OUT_DIR"), "/user.rs"));
   }
   ```

## Usage

In your Loco controller:

```rust
use loco_rw::prelude::*;
use loco_protobuf::Protobuf;
use crate::user::{UserRequest, UserResponse};

async fn create_user(
    Protobuf(req): Protobuf<UserRequest>,
) -> Result<Protobuf<UserResponse>> {
    println!("Received: {:?}", req);
    
    let res = UserResponse {
        id: req.id,
        name: "Loco User".to_string(),
    };
    
    Ok(Protobuf(res))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/user", post(create_user))
}
```

## Testing

You can use `reqwest` or any HTTP client to test. Set `Content-Type: application/protobuf` and send the binary payload.
