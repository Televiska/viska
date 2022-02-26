#[macro_use]
macro_rules! p {
    ($variable:expr) => {
        panic!("{:?} deliberatly panicked, but it shouldn't", $variable)
    };
}

pub mod common;
pub mod unit_tests;
