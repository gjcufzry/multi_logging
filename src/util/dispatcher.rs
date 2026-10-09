use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use crate::{
    fmt::default::DefaultFormatter,
    logger::Logger,
    sink::Sink,
    util::pool::{ObjectPool, ThreadPool},
};

/// 全局调度器实例。
pub static GLOBAL_DISPATCHER: LazyLock<Dispatcher> = LazyLock::new(Dispatcher::new);

/// 全局调度器。
pub struct Dispatcher {
    pub(crate) thread_pool: ThreadPool,
    pub(crate) object_pool: ObjectPool,
    pub(crate) loggers: RwLock<HashMap<Box<str>, Arc<dyn Logger>>>,
    pub(crate) sinks: RwLock<HashMap<Box<str>, Arc<dyn Sink>>>,
    pub(crate) global_formatter: Arc<DefaultFormatter>,
}

impl Dispatcher {
    pub(crate) fn new() -> Self {
        Self {
            thread_pool: ThreadPool::new(),
            object_pool: ObjectPool::new(),
            loggers: RwLock::new(HashMap::new()),
            sinks: RwLock::new(HashMap::new()),
            global_formatter: Arc::new(DefaultFormatter::default()),
        }
    }
}
