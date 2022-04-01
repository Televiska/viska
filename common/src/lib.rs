extern crate envconfig_derive;

mod config;
pub use config::Config;

use once_cell::sync::Lazy;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(config::Config::default()));

pub use async_trait;
pub use bytes;
pub use chrono;
pub use delegate;
pub use futures;
pub use futures_util;
pub use ipnetwork;
pub use log;
pub use md5;
pub use once_cell;
pub use pnet;
pub use pretty_env_logger;
pub use rand;
pub use rand_chacha;
pub use rsip;
//pub use rsip_dns;
pub use tokio;
pub use tokio_util;
pub use uuid;
