#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();

    tokio::spawn(async move {
        server::tcp::start().await;
    }).await;
}
