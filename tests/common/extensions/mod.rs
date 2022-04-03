mod models;
mod rsip_ext;
pub mod transaction_ext;
mod uri_ext;

pub use self::models::{TransactionLayerMsgExt, TransportLayerMsgExt};
pub use uri_ext::{HostWithPortExt, UriExt};

pub trait Randomized: Sized {
    fn default() -> Self;
    fn randomized() -> Self {
        Self::default()
    }
}

pub trait TryClone: Sized {
    type Error: std::fmt::Debug;
    fn try_clone(&self) -> Result<Self, Self::Error>;
    fn try_clone_from(&mut self, other: &Self) -> Result<(), Self::Error> {
        other.try_clone().map(|new| *self = new)
    }
}
