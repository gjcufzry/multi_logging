//!  同步 [`Logger`] 实现。

use std::sync::{Arc, RwLock};

use crossbeam::atomic::AtomicCell;

use crate::{
    logger::Logger,
    sink::Sink,
    util::errors::{LoggerError, LoggerErrorKind},
};

/// 同步的 [`Logger`] 实现。
///
/// # Example
/// ```
/// use multi_logging::logger::SyncLogger;
/// use multi_logging::sink::NullSink;
/// use multi_logging::log::{self, LevelFilter};
/// use std::sync::Arc;
///
/// let logger = SyncLogger::builder()
///     .name("async logger")
///     .level(LevelFilter::Trace)                  // 过滤等级。
///     .add_sink(Arc::new(NullSink::new("null")))  // 添加 sink。
///     .error_handle(|_| {                         // 设置错误回调。
///         println!("Error!");
///         false
///     })
///     .build();
///
/// log::info!(logger: logger, "Hello world!");
/// ```
pub struct SyncLogger {
    name: Box<str>,
    max_level: AtomicCell<log::LevelFilter>,
    sinks: RwLock<Vec<Arc<dyn Sink>>>,
    error_handle: Option<fn(LoggerError) -> bool>,
}

impl Logger for SyncLogger {
    fn set_level(&self, level: log::LevelFilter) {
        self.max_level.store(level);
    }

    fn add_sink(&self, sink: std::sync::Arc<dyn Sink>) {
        self.sinks.write().unwrap().push(sink);
    }

    fn remove_sink(&self, name: String) {
        let mut sinks = self.sinks.write().unwrap();
        for (idx, sink) in sinks.iter().enumerate() {
            if sink.name() == name {
                sinks.swap_remove(idx);
                return;
            }
        }
        self.handle_error(LoggerError::new(LoggerErrorKind::SinkNameNotFound));
    }

    fn enabled(&self, level: log::Level, _target: &str) -> bool {
        self.max_level.load() >= level
    }

    fn log(&self, record: crate::util::Record) {
        for sink in self.sinks.read().unwrap().iter() {
            let _ = sink.log(&record).map_err(|_| {
                self.handle_error(LoggerError::new(LoggerErrorKind::LogError));
            });
        }
    }

    fn flush(&self) {
        self.sinks.read().unwrap().iter().for_each(|s| {
            let _ = s.flush().map_err(|_| {
                self.handle_error(LoggerError::new(LoggerErrorKind::FlushError));
            });
        });
    }

    fn flush_and_wait(&self) {
        self.flush();
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl SyncLogger {
    /// 获取 [`SyncLogger`] 的构造器。
    #[inline]
    pub fn builder() -> SyncLoggerBuilder {
        SyncLoggerBuilder::new()
    }

    #[inline(always)]
    fn handle_error(&self, error: LoggerError) {
        if let Some(handle) = self.error_handle {
            assert!(handle(error));
        }
    }
}

pub struct SyncLoggerBuilder {
    name: Box<str>,
    max_level: log::LevelFilter,
    sinks: Vec<Arc<dyn Sink>>,
    error_handle: Option<fn(LoggerError) -> bool>,
}

impl SyncLoggerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            name: crate::util::DEFAULT_NULL_NAME.into(),
            max_level: log::LevelFilter::Off,
            sinks: Vec::new(),
            error_handle: None,
        }
    }

    /// 设置 logger 名字。
    #[inline]
    pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.name = name.as_ref().into();
        self
    }

    /// 设置过滤等级。
    ///
    /// 默认为 [LevelFilter::Off](log::LevelFilter::Off)。
    #[inline]
    pub fn level(&mut self, level: log::LevelFilter) -> &mut Self {
        self.max_level = level;
        self
    }

    /// 添加一个 sink。
    #[inline]
    pub fn add_sink(&mut self, sink: Arc<dyn Sink>) -> &mut Self {
        self.sinks.push(sink);
        self
    }

    /// 一次性添加多个 sink。
    #[inline]
    pub fn add_sinks(&mut self, sinks: impl IntoIterator<Item = Arc<dyn Sink>>) -> &mut Self {
        self.sinks.extend(sinks);
        self
    }

    /// 设置错误回调。
    #[inline]
    pub fn error_handle(&mut self, handle: fn(LoggerError) -> bool) -> &mut Self {
        self.error_handle = Some(handle);
        self
    }

    /// 构建 [`SyncLogger`]。
    #[inline]
    pub fn build(&mut self) -> SyncLogger {
        SyncLogger {
            name: self.name.clone(),
            max_level: AtomicCell::new(self.max_level),
            sinks: RwLock::new(self.sinks.clone()),
            error_handle: self.error_handle,
        }
    }
}

impl Default for SyncLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

include!("./impl_trait_macros.rs");

impl_log_trait!(SyncLogger);
