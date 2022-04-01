mod common;
pub mod requests;
pub mod responses;

pub trait RandomizedBuilder {
    type Item;

    fn build(self) -> Self::Item;
}

pub mod prelude {
    pub use super::common::*;
    pub use super::requests;
    pub use super::responses;
    pub use super::RandomizedBuilder;
    pub use crate::common::extensions::*;
    pub use crate::common::factories;
}
