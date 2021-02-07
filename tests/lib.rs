#![allow(dead_code)]
#![allow(unused)]
#[macro_use]
pub extern crate diesel_migrations;

#[macro_use]
macro_rules! as_any {
    ($variable:expr, $type:path) => {
        match $variable.as_any().downcast_ref::<$type>() {
            Some(concrete_type) => concrete_type,
            None => {
                panic!("cant't cast value!");
            }
        };
    };
}

#[macro_use]
macro_rules! as_downcasted {
    ($variable:expr, $core:ident, $transaction:ident, $transport:ident, $core_type:path, $transaction_type:path, $transport_type:path) => {
        let core = $variable.core.clone();
        let $core = as_any!(core, $core_type);

        let transaction = $variable.transaction.clone();
        let $transaction = as_any!(transaction, $transaction_type);

        let transport = $variable.transport.clone();
        let $transport = as_any!(transport, $transport_type);
    };
}

#[macro_use]
macro_rules! p {
    ($variable:expr) => {
        panic!("{:?} deliberatly panicked, but it shouldn't", $variable)
    };
}

pub mod common;
//pub mod integration;
pub mod unit_tests;
