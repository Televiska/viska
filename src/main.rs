#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();

    /*
    let tcp = tokio::spawn(async move {
        server::tcp::start().await;
    });
    */

    let udp = tokio::spawn(async move {
        server::udp::start()
            .await
            .expect("failed to start udp server");
    });

    tokio::try_join!(udp).expect("try join failed");
}
