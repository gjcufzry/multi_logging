//! 日志的用户侧接口。

pub mod async_logger;
mod multi;
pub mod sync_logger;

use std::{any::Any, sync::Arc};

use crate::sink::Sink;

#[doc(inline)]
pub use async_logger::AsyncLogger;
#[doc(inline)]
pub use multi::MultiLogger;
#[doc(inline)]
pub use sync_logger::SyncLogger;

pub trait Logger: Sync + Send + log::Log + Any {
    /// 添加一个 [`Sink`] 实例。
    fn add_sink(&self, sink: Arc<dyn Sink>);

    /// 如果存在，则删除 name 对应的 sink。
    fn remove_sink(&self, name: String);

    /// 打印一次日志。
    fn log_once(&self, record: crate::util::Record);

    /// 刷新缓冲区并同步等待。
    ///
    /// 在 [`SyncLogger`] 中，这等价于 [`Logger::flush`]，而在 [`AsyncLogger`] 中，
    /// 这将等待io线程完成刷新。
    fn flush_and_wait(&self);

    /// 该 logger 的名字。
    fn name(&self) -> &str;
}

/// [`AsyncLogger`] 的默认日志缓冲区大小。
pub const DEFAULT_CHANNEL_SIZE: usize = 1024;

impl<T: Logger + ?Sized> Logger for Arc<T> {
    fn add_sink(&self, sink: Arc<dyn Sink>) {
        (**self).add_sink(sink);
    }

    fn remove_sink(&self, name: String) {
        (**self).remove_sink(name);
    }

    fn log_once(&self, record: crate::util::Record) {
        (**self).log_once(record);
    }

    fn flush_and_wait(&self) {
        (**self).flush_and_wait();
    }

    fn name(&self) -> &str {
        (**self).name()
    }
}

impl<T: Logger + ?Sized> Logger for Box<T> {
    fn add_sink(&self, sink: Arc<dyn Sink>) {
        (**self).add_sink(sink);
    }

    fn remove_sink(&self, name: String) {
        (**self).remove_sink(name);
    }
    fn log_once(&self, record: crate::util::Record) {
        (**self).log_once(record);
    }

    fn flush_and_wait(&self) {
        (**self).flush_and_wait();
    }

    fn name(&self) -> &str {
        (**self).name()
    }
}
