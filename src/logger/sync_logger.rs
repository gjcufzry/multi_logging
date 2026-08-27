//!  同步 [`Logger`] 实现。

use std::sync::{Arc, RwLock};

use crossbeam::atomic::AtomicCell;

use crate::{logger::Logger, sink::Sink};

/// 同步的 [`Logger`] 实现。
pub struct SyncLogger {
    name: Box<str>,
    max_level: AtomicCell<log::LevelFilter>,
    sinks: RwLock<Vec<Arc<dyn Sink>>>,
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
    }

    fn enabled(&self, level: log::Level, _target: &str) -> bool {
        self.max_level.load() >= level
    }

    fn log(&self, record: crate::util::Record) {
        for sink in self.sinks.read().unwrap().iter() {
            let _ = sink.log(&record);
        }
    }

    fn flush(&self) {
        self.sinks.read().unwrap().iter().for_each(|s| {
            let _ = s.flush();
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
    #[inline]
    pub fn builder() -> SyncLoggerBuilder {
        SyncLoggerBuilder::new()
    }
}

pub struct SyncLoggerBuilder {
    name: Box<str>,
    max_level: log::LevelFilter,
    sinks: Vec<Arc<dyn Sink>>,
}

impl SyncLoggerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            name: crate::util::DEFAULT_NULL_NAME.into(),
            max_level: log::LevelFilter::Off,
            sinks: Vec::new(),
        }
    }

    #[inline]
    pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.name = name.as_ref().into();
        self
    }

    #[inline]
    pub fn level(&mut self, level: log::LevelFilter) -> &mut Self {
        self.max_level = level;
        self
    }

    #[inline]
    pub fn add_sink(&mut self, sink: Arc<dyn Sink>) -> &mut Self {
        self.sinks.push(sink);
        self
    }

    #[inline]
    pub fn add_sinks(&mut self, sinks: impl IntoIterator<Item = Arc<dyn Sink>>) -> &mut Self {
        self.sinks.extend(sinks);
        self
    }

    #[inline]
    pub fn build(&mut self) -> SyncLogger {
        SyncLogger {
            name: self.name.clone(),
            max_level: AtomicCell::new(self.max_level),
            sinks: RwLock::new(self.sinks.clone()),
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
