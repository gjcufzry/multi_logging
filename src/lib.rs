#![doc = include_str!("../README.md")]

pub mod fmt;
pub mod global;
pub mod logger;
pub mod sink;
pub(crate) mod util;

pub use log;

pub use crate::util::Record;

pub mod prelude {
    pub use crate::sink::Sink;
    pub use crate::logger::Logger;
    pub use crate::fmt::Formatter;
}
