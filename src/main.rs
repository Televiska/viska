fn main() {}

//TODO: this should be moved in a different/outside crate
/*
use sip_server::{
    transaction::Transaction,
    transport::Transport,
    tu::elements::{Capabilities, Registrar, UserAgent},
};

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    let _ = common::Config::default();

    let (handlers, receivers) = models::channels_builder();
    let _ = UserAgent::new(
        handlers.clone(),
        receivers.tu,
        Registrar::new(handlers.clone()),
        Capabilities::new(handlers.clone()),
    );

    let _ = Transaction::new(handlers.clone(), receivers.transaction);

    let _ = Transport::new(handlers.clone(), receivers.transport);

    tokio::spawn(async move {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(4000))
        }
    })
    .await
    .expect("sleeping");
}
*/
