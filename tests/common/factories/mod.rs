mod common;
pub mod models;
pub mod requests;
pub mod responses;

//TODO: not really used yet
pub trait RandomizedBuilder {
    type Item;

    fn build(self) -> Self::Item;
}

pub mod prelude {
    pub use super::common::*;
    pub use super::requests;
    pub use super::responses;
    pub use super::RandomizedBuilder;
    pub use crate::common::factories as factories;
}
