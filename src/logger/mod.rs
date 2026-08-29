//! 日志的用户侧接口。

pub mod async_logger;
pub mod sync_logger;

use std::sync::Arc;

use crate::sink::Sink;

pub use async_logger::AsyncLogger;
pub use sync_logger::SyncLogger;

pub trait Logger: Sync + Send {
    /// 设置 logger 的过滤等级。
    fn set_level(&self, level: log::LevelFilter);

    /// 添加一个 [`Sink`] 实例。
    fn add_sink(&self, sink: Arc<dyn Sink>);

    /// 如果存在，则删除 name 对应的 sink。
    fn remove_sink(&self, name: String);

    /// 获取是否应该显示某一个 log。
    fn enabled(&self, level: log::Level, target: &str) -> bool;

    /// 打印一次日志。
    fn log(&self, record: crate::util::Record);

    /// 刷新。
    fn flush(&self);

    /// 刷新缓冲区并同步等待。
    ///
    /// 在 [`SyncLogger`] 中，这等价于 [`Logger::flush`]，而在 [`AsyncLogger`] 中，
    /// 这将等待io线程完成刷新。
    fn flush_and_wait(&self);

    /// 该 logger 的名字。
    fn name(&self) -> &str;
}

include!("./impl_trait_macros.rs");

impl_log_trait!(dyn Logger);

/// [`AsyncLogger`] 的默认日志缓冲区大小。
pub const DEFAULT_CHANNEL_SIZE: usize = 1024;
