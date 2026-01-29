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

---

# Loco ProtoBuf 詳細解説ガイド

このセクションでは、`Loco ProtoBuf` クレートの機能、導入方法、および使用方法について、詳細に解説します。

## プロジェクトの概要

**Loco ProtoBuf** は、Webフレームワーク [Loco.rs](https://loco.rs) (Axumベース) アプリケーションにおいて、**Protocol Buffers (Protobuf)** 形式のデータを容易に扱うための統合ライブラリです。

通常、Web APIでは JSON が広く使われますが、Protobuf を使用することで以下のメリットが得られます：
*   **データサイズの削減**: バイナリ形式のため、JSONよりも通信量が少なくなります。
*   **型安全性**: `.proto` ファイルによるスキーマ定義に基づき、厳密な型チェックが行われます。
*   **高速な処理**: パース（デシリアライズ）および生成（シリアライズ）が高速です。

このクレートは、Loco/Axum の生態系に Protobuf を透過的に組み込むための「接着剤」の役割を果たします。

## 主な機能 (Features)

Loco ProtoBuf は、Axum の仕組み（Extractor と Response）を利用して実装されています。

### 1. `Protobuf<T>` エクストラクター
HTTPリクエストのボディ（`Content-Type: application/protobuf`）を受け取り、自動的にデシリアライズして Rust の構造体（`T`）に変換します。
開発者は、コントローラーの引数に `Protobuf(req): Protobuf<UserRequest>` のように記述するだけで、パース済みのデータを受け取ることができます。

### 2. `Protobuf<T>` レスポンス
Rust の構造体をレスポンスとして返す際、`Protobuf(res)` でラップすることで、自動的に Protobuf バイナリ形式にシリアライズし、適切な `Content-Type` ヘッダーを設定してクライアントに返却します。

## 導入手順詳解 (Installation & Setup)

導入には、依存関係の追加とビルドプロセスの設定の2段階が必要です。

### ステップ 1: 依存関係の定義 (`Cargo.toml`)

```toml
[dependencies]
loco-protobuf = "0.1"  # 本クレート
prost = "0.13"         # Rust用Protobufランタイム
axum = "0.8"           # Webフレームワーク本体

[build-dependencies]
prost-build = "0.13"   # `.proto`ファイルをコンパイルするためのビルドツール
```

> **注意**: `prost` と `prost-build` のバージョンは合わせることを推奨します。

### ステップ 2: スキーマ定義

`proto/` ディレクトリを作成し、その中に `.proto` ファイル（例: `user.proto`）を配置します。これがAPIの契約（コントラクト）となります。

```protobuf
syntax = "proto3";
package user;

// リクエストの型定義
message UserRequest {
  string id = 1;
}

// レスポンスの型定義
message UserResponse {
  string id = 1;
  string name = 2;
}
```

### ステップ 3: ビルドスクリプトの設定 (`build.rs`)

Rustのコンパイル前に `.proto` ファイルを Rust コードに変換するため、プロジェクトルートに `build.rs` を作成します。

```rust
fn main() {
    // proto/user.proto をコンパイルし、結果を OUT_DIR に出力する
    prost_build::compile_protos(&["proto/user.proto"], &["proto"]).unwrap();
}
```

> **環境構築のヒント**: ローカル環境に `protoc` コマンドを入れたくない場合は、`protoc-bin-vendored` クレートを `build-dependencies` に追加することで、ビルド時に自動的にバイナリを確保できます。

### ステップ 4: 生成コードの取り込み

`build.rs` によって生成された Rust コードは、通常 `target/debug/build/.../out/` などの深い階層に出力されます。これをアプリ内で使えるように、`include!` マクロを使って読み込みます。

```rust
// src/lib.rs または src/proto.rs など
pub mod user {
    // env!("OUT_DIR") はビルド時の出力ディレクトリパスを展開します
    include!(concat!(env!("OUT_DIR"), "/user.rs"));
}
```

## 実装例 (Usage)

Loco のコントローラーでの使用例です。JSON を扱うのとほぼ同じ感覚で Protobuf を扱えます。

```rust
use loco_rs::prelude::*;
use loco_protobuf::Protobuf;
// 生成された構造体をインポート
use crate::user::{UserRequest, UserResponse};

async fn create_user(
    // リクエストボディを UserRequest 構造体として受け取る
    Protobuf(req): Protobuf<UserRequest>,
) -> Result<Protobuf<UserResponse>> {
    // ログ出力（req.id は既にString型として利用可能）
    info!(id = %req.id, "received user request");
    
    // レスポンスデータの構築
    let res = UserResponse {
        id: req.id,
        name: "Loco User".to_string(),
    };
    
    // Protobufでラップして返す
    Ok(Protobuf(res))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/user", post(create_user))
}
```

## テスト方法 (Testing)

作成した API をテストするには、HTTPクライアントでバイナリデータを送信する必要があります。

*   **ヘッダー**: `Content-Type: application/protobuf` を必ず設定してください。
*   **ボディ**: 言語ごとの Protobuf ライブラリでシリアライズしたバイナリデータを送信します。

`curl` などでテストする場合は、バイナリファイルを直接指定するなどの工夫が必要です。

## 開発用ユーティリティ

リポジトリには `scripts/get-pr-review-comments.sh` が含まれています。これは GitHub CLI (`gh`) をラップしたスクリプトで、プルリクエストについたレビューコメントをローカルのターミナルで一覧表示するために使用されます。

```bash
# 現在のPRまたは指定したPRのレビューコメントを取得
./scripts/get-pr-review-comments.sh [PR番号]
```
