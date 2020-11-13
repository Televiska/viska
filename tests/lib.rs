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

pub mod common;
//pub mod integration;
pub mod unit_tests;
