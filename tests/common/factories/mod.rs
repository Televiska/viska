pub mod common;
pub mod headers;
pub mod models;
pub mod requests;
pub mod responses;

//TODO: not really used yet
pub trait RandomizedBuilder {
    type Item;

    fn build(self) -> Self::Item;
}
