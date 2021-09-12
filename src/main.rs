use sip_server::{
    core::{Capabilities, Core, Processor, Registrar},
    SipBuilder, Transaction, Transport,
};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    let config = common::Config::default();

    println!("{:?}", config);

    if std::env::args().len() == 1 {
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
    } else {
        tasks::run_task().await.expect("run task failed");
    }
}
