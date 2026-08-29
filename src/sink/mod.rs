//! 日志输出支持。

pub mod base_sink;
pub mod file_sink;
pub mod hook_sink;
pub mod null_sink;
pub mod stderr_sink;
pub mod stdout_sink;

pub use file_sink::{FileSinkMT, FileSinkST};
pub use hook_sink::HookSink;
pub use null_sink::NullSink;
pub use stderr_sink::{StderrSinkMT, StderrSinkST};
pub use stdout_sink::{StdoutSinkMT, StdoutSinkST};

use crate::util::{Record, errors::SinkError};

/// 用于输出格式化之后的日志到指定的位置。
pub trait Sink: Send + Sync {
    /// 将仅格式化之后的日志以及元数据输出。
    fn log(&self, record: &Record) -> SinkResult<()>;

    /// 刷新缓冲区。
    fn flush(&self) -> SinkResult<()>;

    /// 获取 [`Sink`] 在注册时的名字。
    fn name(&self) -> &str;

    /// 更新日志过滤等级。
    fn set_level(&self, level: log::LevelFilter);
}

/// [`Sink`] 的 [`Result`]返回值 别名。
pub(crate) type SinkResult<Ret> = Result<Ret, SinkError>;

/// 默认缓冲区大小。
pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 16;
