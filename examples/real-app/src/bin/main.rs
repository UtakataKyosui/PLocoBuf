use loco_rs::cli;
use migration::Migrator;
use real_app::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    if let Ok(true) = real_app::loco_protobuf::gen::handle() {
        return Ok(());
    }
    cli::main::<App, Migrator>().await
}
