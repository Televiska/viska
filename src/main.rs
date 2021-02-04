use sip_server::{Core, SipBuilder, Transaction, Transport};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    common::Config::verify();

    let manager = SipBuilder::new::<Core, Transaction, Transport>().expect("sip manager failed");
    manager.run().await;

    tokio::spawn(async move {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(4000))
        }
    })
    .await
    .expect("sleeping");
}
