use sip_server::{
    core::{Capabilities, Core, Processor, Registrar},
    SipBuilder, Transaction, Transport,
};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    let config = common::Config::new();

    println!("{:?}", config);

    let manager =
        SipBuilder::new::<Core<Processor<Registrar, Capabilities>>, Transaction, Transport>()
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
