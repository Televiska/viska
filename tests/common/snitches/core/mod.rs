mod capabilities;
mod core;
mod registrar;
mod dialogs;

pub use capabilities::CapabilitiesSnitch;
pub use self::core::{CorePanic, CoreSnitch};
pub use registrar::RegistrarSnitch;
pub use dialogs::DialogsEmptySnitch;

use common::async_trait::async_trait;
use models::transport::RequestMsg;
use sip_server::{ReqProcessor, Error, SipManager};
use std::any::Any;
use std::sync::Weak;

#[derive(Debug)]
pub struct ReqProcessorPanic;

#[async_trait]
impl ReqProcessor for ReqProcessorPanic {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self
    }

    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        p!(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
