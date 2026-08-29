//! 异步 [`Logger`] 实现。

use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    sync::{Arc, Barrier},
};

use crossbeam::{
    atomic::AtomicCell,
    channel::{Receiver, Sender},
};

use crate::{
    logger::Logger,
    sink::Sink,
    util::{
        Record,
        errors::{LoggerError, LoggerErrorKind},
    },
};

/// 异步的 [`Logger`] 实现。
///
/// # Example
/// ```
/// use multi_logging::logger::AsyncLogger;
/// use multi_logging::sink::NullSink;
/// use multi_logging::log::{self, LevelFilter};
/// use std::sync::Arc;
///
/// let logger = AsyncLogger::builder()
///     .name("async logger")
///     .level(LevelFilter::Trace)                  // 过滤等级。
///     .bound(true)                                // 有界队列。
///     .chanel_size(1024)                          // 队列大小。
///     .add_sink(Arc::new(NullSink::new("null")))  // 添加 sink。
///     .drop_when_blocked(true)                    // 在队列阻塞时是否丢弃日志。
///     .error_handle(|_| {                         // 设置错误回调。
///         println!("Error!");
///         false
///     })
///     .build();
///
/// log::info!(logger: logger, "Hello world!");
/// ```
pub struct AsyncLogger {
    name: Box<str>,
    max_level: AtomicCell<log::LevelFilter>,
    sender: Sender<LogCommand>,
    drop_when_blocked: bool,
}

impl Logger for AsyncLogger {
    fn set_level(&self, level: log::LevelFilter) {
        self.max_level.store(level);
    }

    fn add_sink(&self, sink: Arc<dyn Sink>) {
        self.send_command(LogCommand::AddSink(sink));
    }

    fn remove_sink(&self, name: String) {
        self.send_command(LogCommand::RemoveSink(name));
    }

    fn enabled(&self, level: log::Level, _target: &str) -> bool {
        self.max_level.load() >= level
    }

    fn log(&self, record: crate::util::Record) {
        self.send_command(LogCommand::LogOnce(record));
    }

    fn flush(&self) {
        self.send_command(LogCommand::Flush);
    }

    fn flush_and_wait(&self) {
        let bar = Arc::new(Barrier::new(2));
        self.send_command(LogCommand::FlushAndWait(bar.clone()));
        let _ = bar.wait();
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct AsyncLoggerBuilder {
    /// 名字。
    pub name: Box<str>,
    /// 过滤等级。
    pub level: log::LevelFilter,
    /// 在队列满时是否阻塞。
    pub bound: bool,
    /// 队列大小。
    pub chanel_size: usize,
    /// 绑定的 [`Sink`] 对象。
    pub sinks: Vec<Arc<dyn Sink>>,
    /// 错误处理函数。
    pub error_handle: Option<fn(LoggerError) -> bool>,
    /// 在队列阻塞时是否丢弃日志。
    pub drop_when_blocked: bool,
}

impl AsyncLogger {
    #[inline]
    pub fn builder() -> AsyncLoggerBuilder {
        AsyncLoggerBuilder::new()
    }

    #[inline(always)]
    fn send_command(&self, command: LogCommand) {
        if self.drop_when_blocked {
            let _ = self.sender.try_send(command);
        } else {
            let _ = self.sender.send(command);
        }
    }
}

impl AsyncLoggerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            name: crate::util::DEFAULT_NULL_NAME.into(),
            level: log::LevelFilter::Off,
            bound: false,
            chanel_size: super::DEFAULT_CHANNEL_SIZE,
            sinks: Vec::new(),
            error_handle: None,
            drop_when_blocked: true,
        }
    }

    /// 设置 [`AsyncLogger`] 的名字。
    ///
    /// 默认为 "__NULL_"。
    #[inline]
    pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.name = name.as_ref().into();
        self
    }

    /// 设置 [`AsyncLogger`] 的日志等级。
    ///
    /// 默认为 [`log::LevelFilter::Off`]。
    #[inline]
    pub fn level(&mut self, level: log::LevelFilter) -> &mut Self {
        self.level = level;
        self
    }

    /// 异步队列是否有界。
    #[inline]
    pub fn bound(&mut self, bound: bool) -> &mut Self {
        self.bound = bound;
        self
    }

    /// 异步队列大小。
    #[inline]
    pub fn chanel_size(&mut self, size: usize) -> &mut Self {
        self.chanel_size = size;
        self
    }

    /// 添加多个 [`Sink`] 实例。
    #[inline]
    pub fn add_sinks(&mut self, sinks: impl IntoIterator<Item = Arc<dyn Sink>>) -> &mut Self {
        self.sinks.extend(sinks);
        self
    }

    /// 添加单个 [`Sink`] 实例。
    #[inline]
    pub fn add_sink(&mut self, sink: Arc<dyn Sink>) -> &mut Self {
        self.sinks.push(sink);
        self
    }

    /// 错误处理函数。
    ///
    /// 返回值为 `false` 时，程序将会 Panic。
    #[inline]
    pub fn error_handle(&mut self, handle: fn(LoggerError) -> bool) -> &mut Self {
        self.error_handle = Some(handle);
        self
    }

    /// 在队列阻塞时是否丢弃日志。
    #[inline]
    pub fn drop_when_blocked(&mut self, option: bool) -> &mut Self {
        self.drop_when_blocked = option;
        self
    }

    /// 最终构造。
    #[inline]
    pub fn build(&self) -> AsyncLogger {
        let tx = AsyncHandleThread::make_thread(self);
        AsyncLogger {
            name: self.name.clone(),
            max_level: AtomicCell::new(self.level),
            sender: tx,
            drop_when_blocked: self.drop_when_blocked,
        }
    }
}

impl Default for AsyncLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) enum LogCommand {
    /// 执行一次日志输出
    LogOnce(Record),
    /// 添加一个 Sink 。
    AddSink(Arc<dyn Sink>),
    /// 使用名字寻找并删除Sink
    RemoveSink(String),
    /// 刷新缓冲区。
    Flush,
    /// 刷新所有缓冲区并退出线程。
    Exit,
    /// 刷新缓冲区并使用信号量等待线程完成。
    FlushAndWait(Arc<Barrier>),
}

/// 用于在异步线程中保存所需数据以及处理任务。
pub(crate) struct AsyncHandleThread {
    sinks: Vec<Arc<dyn Sink>>,
    command_buf: VecDeque<LogCommand>,
    rec: Receiver<LogCommand>,
    error_handle: Option<fn(LoggerError) -> bool>,
}

impl AsyncHandleThread {
    fn make_thread(builder: &AsyncLoggerBuilder) -> Sender<LogCommand> {
        let (tx, rx) = if builder.bound {
            crossbeam::channel::bounded(builder.chanel_size)
        } else {
            crossbeam::channel::unbounded()
        };
        let sinks = builder.sinks.clone();
        let error_handle = builder.error_handle;

        let handle = std::thread::spawn(move || {
            let mut data = Self {
                sinks,
                command_buf: VecDeque::with_capacity(128),
                rec: rx,
                error_handle,
            };
            data.work();
        });

        // 注册全局句柄。
        crate::util::dispatcher::GLOBAL_DISPATCHER
            .thread_pool
            .spawn(handle, tx.clone());

        tx
    }

    #[inline(always)]
    fn work(&mut self) {
        while let Ok(command) = self.rec.recv() {
            self.command_buf.push_back(command);
            for _ in 0..127 {
                // 本次循环尝试一次性获取 128 条命令。
                let Ok(com) = self.rec.try_recv() else {
                    continue;
                };
                self.command_buf.push_back(com);
            }
            while let Some(command) = self.command_buf.pop_front() {
                match command {
                    LogCommand::LogOnce(record) => self.log_once(record),
                    LogCommand::AddSink(sink) => self.add_sink(sink),
                    LogCommand::RemoveSink(name) => self.remove_sink(&name),
                    LogCommand::Flush => self.flsuh(),
                    LogCommand::Exit => {
                        self.flsuh();
                        return;
                    }
                    LogCommand::FlushAndWait(b) => self.flush_and_wait(b),
                }
            }
        }
    }

    #[inline(always)]
    fn log_once(&self, record: Record) {
        for sink in self.sinks.iter() {
            self.try_handle_error(LoggerErrorKind::LogError, sink.log(&record));
        }
    }

    #[inline(always)]
    fn add_sink(&mut self, sink: Arc<dyn Sink>) {
        self.sinks.push(sink);
    }

    #[inline(always)]
    fn remove_sink(&mut self, name: &str) {
        for (idx, sink) in self.sinks.iter().enumerate() {
            if sink.name() == name {
                self.sinks.swap_remove(idx);
                return;
            }
        }
        self.handle_error(LoggerError::new(LoggerErrorKind::SinkNameNotFound));
    }

    #[inline(always)]
    fn flsuh(&self) {
        for sink in &self.sinks {
            self.try_handle_error(LoggerErrorKind::FlushError, sink.flush());
        }
    }

    #[inline(always)]
    fn flush_and_wait(&self, b: Arc<Barrier>) {
        self.flsuh();
        let _ = b.wait();
    }

    #[inline(always)]
    fn try_handle_error<Base: Debug + Display + 'static>(
        &self,
        kind: LoggerErrorKind,
        error: Result<(), crate::util::errors::BaseError<Base>>,
    ) {
        if let Err(e) = error
            && let Some(handle) = self.error_handle
        {
            assert!(handle(LoggerError::with_source(kind, e)));
        }
    }

    #[inline(always)]
    fn handle_error(&self, error: LoggerError) {
        if let Some(handle) = self.error_handle {
            assert!(handle(error));
        }
    }
}

include!("./impl_trait_macros.rs");

impl_log_trait!(AsyncLogger);
