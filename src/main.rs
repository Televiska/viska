use processor::{Core, SipBuilder, Transaction, Transport};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    common::Config::verify();

    let udp = tokio::spawn(async move {
        let manager =
            SipBuilder::new::<Core, Transaction, Transport>().expect("sip manager failed");

        manager.run().await
    });

    tokio::try_join!(udp).expect("try join failed");
}
