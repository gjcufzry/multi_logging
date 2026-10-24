#![doc = include_str!("../README.md")]

pub mod fmt;
pub mod global;
pub mod logger;
pub mod sink;
pub(crate) mod util;
pub use log;

pub use crate::util::Record;

pub mod prelude {
    pub use crate::fmt::{self, ColorFormatter, DefaultFormatter, Formatter as _};
    pub use crate::logger::{self, AsyncLogger, Logger as _, MultiLogger, SyncLogger};
    pub use crate::sink::{
        FileSinkMT, FileSinkST, FlushSink, HookSink, NullSink, Sink as _, StderrSinkMT,
        StderrSinkST, StdoutSinkST,
    };
}
