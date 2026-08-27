#![doc = include_str!("../README.md")]

pub mod fmt;
pub mod logger;
pub mod sink;
pub(crate) mod util;

pub use crate::util::dispatcher::{
    GLOBAL_DISPATCHER, exit, get_logger, get_sink, register_logger, register_sink, remove_logger,
    remove_sink, set_global_format,
};
pub use log;
