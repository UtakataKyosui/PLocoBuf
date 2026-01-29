# Loco ProtoBuf

A helper crate to easily integrate Protocol Buffers with [Loco.rs](https://loco.rs), offering Axum-compatible extractors and responses.

[Loco.rs](https://loco.rs) (Axumベース) アプリケーションにおいて、Protocol Buffers (Protobuf) を容易に扱うための統合ライブラリです。`application/protobuf` なリクエストの自動デシリアライズや、レスポンスのシリアライズをサポートします。

## Features / 特徴

- **`Protobuf<T>` Extractor**
  - Automatically decodes `application/protobuf` request bodies into your Prost-generated structs.
  - リクエストボディを自動的にデコードし、Prostで生成された構造体に変換します。

- **`Protobuf<T>` Response**
  - Automatically encodes your structs into Protobuf format and sets the `Content-Type: application/protobuf` header.
  - 構造体を自動的にProtobuf形式にエンコードし、適切な `Content-Type` ヘッダーを付与してレスポンスします。

- **Axum 0.8 / Loco 0.16 Support**
  - Built for the latest Loco versions.
  - 最新の Loco バージョンに対応しています。

## Installation / 導入

Add the following to your `Cargo.toml`.
`Cargo.toml` に以下の依存関係を追加してください。

```toml
[dependencies]
loco-protobuf = "0.1"
prost = "0.13"         # Rust Protobuf runtime
axum = "0.8"           # Web framework

[build-dependencies]
prost-build = "0.13"   # Tool to compile .proto files
```

> 💡 **Tip**: dependenciesとbuild-dependenciesで `prost` 関連のバージョンを合わせることを推奨します。

## Setup / セットアップ手順

### 1. Define your Protos (スキーマ定義)

Create a `proto` directory and add your `.proto` files (e.g., `proto/user.proto`).
`proto` ディレクトリを作成し、その中に `.proto` ファイル（例: `user.proto`）を配置します。

```protobuf
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

### 2. Configure Build Script (ビルドスクリプト設定)

Create or update `build.rs` to compile protos before building the project.
プロジェクトルートの `build.rs` を編集し、ビルド時に `.proto` ファイルをコンパイルするように設定します。

```rust
fn main() {
    // Compile proto/user.proto and output to OUT_DIR
    // proto/user.proto をコンパイル
    prost_build::compile_protos(&["proto/user.proto"], &["proto"]).unwrap();
}
```

> **ProTip**: Use [`protoc-bin-vendored`](https://crates.io/crates/protoc-bin-vendored) if you don't want to install the `protoc` compiler manually on your system.
> `protoc` コマンドをローカルにインストールしたくない場合は、`protoc-bin-vendored` クレートを使用すると便利です。

### 3. Include Generated Code (生成コードの読み込み)

In your `src/lib.rs` or `src/proto.rs`, include the generated code.
生成されたRustコードをアプリケーション内で利用できるように `include!` マクロで読み込みます。

```rust
pub mod user {
    // env!("OUT_DIR") expands to the build output directory
    include!(concat!(env!("OUT_DIR"), "/user.rs"));
}
```

## Usage / 使用方法

Use `Protobuf<T>` in your Loco controller to handle requests and responses.
Locoのコントローラーで `Protobuf<T>` を使用して、リクエストとレスポンスを処理します。

```rust
use loco_rs::prelude::*;
use loco_protobuf::Protobuf;
// Import generated structs
// 生成された構造体をインポート
use crate::user::{UserRequest, UserResponse};

async fn create_user(
    // Extract UserRequest from request body
    // リクエストボディから UserRequest を抽出
    Protobuf(req): Protobuf<UserRequest>,
) -> Result<Protobuf<UserResponse>> {
    tracing::info!(id = %req.id, "received user request");
    
    let res = UserResponse {
        id: req.id,
        name: "Loco User".to_string(),
    };
    
    // Return UserResponse as Protobuf binary
    // UserResponse を Protobuf バイナリとして返す
    Ok(Protobuf(res))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/user", post(create_user))
}
```

## Testing / テスト

When testing with `reqwest` or `curl`, ensure you set the correct header and send binary data.
テスト時はヘッダーを正しく設定し、バイナリデータを送信してください。

- **Header**: `Content-Type: application/protobuf`
- **Body**: Binary payload serialized by Protobuf (Protobufでシリアライズされたバイナリデータ)

## Development

### Fetching PR Review Comments

For contributors, a helper script is available to fetch GitHub PR review comments.
開発者向けに、GitHubのPRレビューコメントを取得するスクリプトが用意されています。

```bash
# Fetch comments for the first open PR / 最初のオープンなPRのコメントを取得
./scripts/get-pr-review-comments.sh

# Fetch comments for a specific PR number / 指定したPR番号のコメントを取得
./scripts/get-pr-review-comments.sh 1
```

