use sip_server::{
    core::{Capabilities, Core, Dialogs, Registrar},
    SipBuilder, Transaction, Transport,
};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    let config = common::Config::default();

    println!("{:?}", config);

    let manager =
        SipBuilder::new::<Core<Registrar, Capabilities, Dialogs>, Transaction, Transport>()
            .expect("sip manager failed");
    manager.run().await;

    tokio::spawn(async move {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(4000))
        }
    })
    .await
    .expect("sleeping");
}
