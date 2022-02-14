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
        }
    };
}

#[macro_use]
macro_rules! as_downcasted {
    ($sip_manager:expr, $tu:ident, $transaction:ident, $transport:ident, $tu_type:path, $transaction_type:path, $transport_type:path) => {
        let tu = $sip_manager.tu.clone();
        let $tu = as_any!(tu, $tu_type);

        let transaction = $sip_manager.transaction.clone();
        let $transaction = as_any!(transaction, $transaction_type);

        let transport = $sip_manager.transport.clone();
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
pub mod unit_tests;
