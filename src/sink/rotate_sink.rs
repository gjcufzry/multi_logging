use std::time::{Duration, Instant};

use crossbeam::atomic::AtomicCell;

use crate::sink::Sink;

pub struct RotateSink<T: Sink> {
    inner: T,
    last: AtomicCell<Instant>,
    dur: Duration,
}

impl<T: Sink> Sink for RotateSink<T> {
    #[inline(always)]
    fn log(&self, record: &crate::util::Record) -> super::SinkResult<()> {
        self.inner.log(record)?;
        if self.need_flush() {
            self.flush()?;
        }
        Ok(())
    }

    #[inline(always)]
    fn flush(&self) -> super::SinkResult<()> {
        self.inner.flush()
    }

    #[inline(always)]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[inline(always)]
    fn set_level(&self, level: log::LevelFilter) {
        self.inner.set_level(level);
    }
}

impl<T: Sink> RotateSink<T> {
    #[inline(always)]
    pub fn new(inner: T, dur: Duration) -> Self {
        Self {
            inner,
            last: AtomicCell::new(Instant::now()),
            dur,
        }
    }

    #[inline(always)]
    fn need_flush(&self) -> bool {
        let now = Instant::now();
        if now
            .checked_duration_since(self.last.load())
            .unwrap_or(Duration::ZERO)
            > self.dur
        {
            self.last.store(now);
            true
        } else {
            false
        }
    }
}
