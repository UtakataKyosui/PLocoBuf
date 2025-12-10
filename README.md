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

First, define your Protocol Buffer schema:

**`proto/user.proto`**
```proto
syntax = "proto3";

package user;

message UserRequest {
  string id = 1;
}

message UserResponse {
  string id = 1;
  string name = 2;
}
```

Then, in your Loco controller:

```rust
use loco_rs::prelude::*;
use loco_protobuf::Protobuf;
use crate::user::{UserRequest, UserResponse};

async fn create_user(
    Protobuf(req): Protobuf<UserRequest>,
) -> Result<Protobuf<UserResponse>> {
    info!(id = %req.id, "received user request");
    
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



3.  **Generate Migration from Proto**:
    
    If you have an existing `.proto` file (e.g., `proto/user.proto`), you can automatically generate a Loco DB migration for it:
    
    ```bash
    cargo loco generate migration_from_proto user
    ```
    
    This reads `proto/user.proto`, parses the fields, maps types (e.g., `int64` -> `bigint`), and invokes `cargo loco generate migration` for you.

---

## Generator Commands

`loco-protobuf` provides CLI commands to quickly scaffold ProtoBuf-based APIs.

### 1. `generate proto_model`

Generate `.proto` file, database migration, and auto-migrate:

```bash
cargo loco generate proto_model product name:string! price:double stock:int
```

**Output**: `proto/product.proto`, migration file, database table created

### 2. `generate proto_controller`

Generate controller with ProtoBuf handlers:

```bash
cargo loco generate proto_controller products
```

**Output**: `src/controllers/products.rs` with CRUD endpoints

### 3. `generate proto_scaffold`

Full CRUD scaffold (combines above):

```bash
cargo loco generate proto_scaffold post title:string! content:text
```

**Integration**: Add to `src/bin/main.rs`:
```rust
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // Intercept generator commands
    if loco_protobuf::gen::handle()? {
        return Ok(());
    }
    cli::main::<App, Migrator>().await
}
```

---

## Testing

You can use `reqwest` or any HTTP client to test. Set `Content-Type: application/protobuf` and send the binary payload.

## Development

### Fetching PR Review Comments

For contributors working on pull requests, a helper script is available to fetch and display review comments:

```bash
# Fetch comments for the first open PR
./scripts/get-pr-review-comments.sh

# Fetch comments for a specific PR number
./scripts/get-pr-review-comments.sh 1
```

This script displays:
- PR information (title, author, state)
- Review summaries
- General comments
- Code-specific review comments with file and line information
