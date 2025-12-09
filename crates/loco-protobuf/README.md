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

## CLI Generator (Optional)

You can integrate a `generate protobuf` command into your Loco CLI.

1.  In `src/bin/main.rs`, wrap the execution:

    ```rust
    // src/bin/main.rs
    use my_app::app::App;
    use my_app::loco_protobuf; // Ensure this re-export exists in lib.rs

    #[tokio::main]
    async fn main() -> loco_rs::Result<()> {
        // Intercept generate protobuf command
        if let Ok(true) = loco_protobuf::gen::handle() {
            return Ok(());
        }
        
        loco_rs::cli::main::<App, migration::Migrator>().await
    }
    ```

2.  Run the generator:
    ```bash
    cargo loco generate protobuf user
    # Creates proto/user.proto
    ```

## Testing

You can use `reqwest` or any HTTP client to test. Set `Content-Type: application/protobuf` and send the binary payload.
