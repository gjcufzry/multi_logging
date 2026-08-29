//! 钩子 sink。
use crate::{
    sink::{Sink, null_sink::NullSink},
    util::Record,
};

/// 一个包装类型，可以在 [`Sink`] 实例执行函数前设置回调函数。
///
/// 其名字与其包装的实例相同。
///
/// # Example
/// ```
/// use multi_logging::sink::{NullSink, HookSink};
///
/// let null_sink = NullSink::new("null");
/// let hook_sink = HookSink::builder()
///     .inner(null_sink)                      // 设置内部实例。
///     .on_flush(|| println!("Sink flush!"))  // 设置回调。
///     .build();
/// ```
pub struct HookSink<T: Sink = NullSink> {
    inner: T,
    on_log: Option<fn(&Record)>,
    on_flush: Option<fn()>,
    on_set_level: Option<fn(log::LevelFilter)>,
}

impl<T: Sink> HookSink<T> {
    pub fn builder() -> HookSinkBuilder<T> {
        HookSinkBuilder::default()
    }
}

impl<T: Sink> Sink for HookSink<T> {
    #[inline]
    fn log(&self, record: &Record) -> super::SinkResult<()> {
        if let Some(func) = self.on_log {
            func(record);
        }
        self.inner.log(record)
    }

    #[inline]
    fn flush(&self) -> super::SinkResult<()> {
        if let Some(func) = self.on_flush {
            func();
        }
        self.inner.flush()
    }

    #[inline]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[inline]
    fn set_level(&self, level: log::LevelFilter) {
        if let Some(func) = self.on_set_level {
            func(level);
        }
        self.inner.set_level(level);
    }
}

pub struct HookSinkBuilder<T: Sink> {
    inner: Option<T>,
    on_log: Option<fn(&Record)>,
    on_flush: Option<fn()>,
    on_set_level: Option<fn(log::LevelFilter)>,
}

impl<T: Sink> HookSinkBuilder<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: None,
            on_log: None,
            on_flush: None,
            on_set_level: None,
        }
    }

    /// 设置 [`HookSink`] 的内部包装类型。
    #[inline]
    pub fn inner(&mut self, inner: T) -> &mut Self {
        self.inner = Some(inner);
        self
    }

    /// 设置在调用 [`log`](Sink::log) 时的回调。
    #[inline]
    pub fn on_log(&mut self, func: fn(&Record)) -> &mut Self {
        self.on_log = Some(func);
        self
    }

    /// 设置在调用 [`flush`](Sink::flush) 时的回调。
    #[inline]
    pub fn on_flush(&mut self, func: fn()) -> &mut Self {
        self.on_flush = Some(func);
        self
    }

    /// 设置在调用 [`set_level`](Sink::set_level) 时的回调。
    #[inline]
    pub fn on_set_level(&mut self, func: fn(log::LevelFilter)) -> &mut Self {
        self.on_set_level = Some(func);
        self
    }

    /// 构造 [`HookSink`]
    #[inline]
    pub fn build(&mut self) -> HookSink<T> {
        HookSink {
            inner: self.inner.take().expect("Sink instance not set yet."),
            on_log: self.on_log,
            on_flush: self.on_flush,
            on_set_level: self.on_set_level,
        }
    }
}

impl<T: Sink> Default for HookSinkBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
