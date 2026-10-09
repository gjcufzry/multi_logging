use std::{
    sync::RwLock,
    time::{Duration, SystemTime},
};

use crate::sink::Sink;

/// 超时后可自动刷新的包装sink。
pub struct FlushSink<T: Sink> {
    timeout: Duration,
    time_stamp: RwLock<SystemTime>,
    inner: T,
}

impl<T: Sink> FlushSink<T> {
    #[inline]
    pub const fn new(sink: T, timeout: Duration) -> Self {
        Self {
            timeout,
            time_stamp: RwLock::new(SystemTime::UNIX_EPOCH),
            inner: sink,
        }
    }
}

impl<T: Sink> Sink for FlushSink<T> {
    #[inline]
    fn log(&self, record: &crate::Record) -> super::SinkResult<()> {
        let res = self.inner.log(record);
        let guard = self.time_stamp.read().unwrap();
        if record
            .time_stamp()
            .duration_since(*guard)
            .map(|s| s > self.timeout)
            .unwrap_or(false)
        {
            drop(guard);
            *self.time_stamp.write().unwrap() = record.time_stamp();
            self.flush()?;
        }
        res
    }

    #[inline]
    fn flush(&self) -> super::SinkResult<()> {
        self.inner.flush()
    }

    #[inline]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[inline]
    fn set_level(&self, level: log::LevelFilter) {
        self.inner.set_level(level);
    }
}
