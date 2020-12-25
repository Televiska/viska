mod common;
mod models_ext;
pub mod requests;
pub mod responses;
mod rsip_ext;

pub trait RandomizedBuilder {
    type Item;

    fn build(self) -> Self::Item;
}

pub trait Randomized: Sized {
    fn default() -> Self;
    fn randomized() -> Self {
        Self::default()
    }
}

pub mod prelude {
    pub use super::common::*;
    pub use super::requests;
    pub use super::responses;
    pub use super::Randomized;
    pub use super::RandomizedBuilder;
    pub use crate::common::factories;
}
