use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use crate::{
    fmt::{Formatter, default::DefaultFormater},
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
    pub(crate) global_formatter: Arc<dyn Formatter>,
}

impl Dispatcher {
    pub(crate) fn new() -> Self {
        Self {
            thread_pool: ThreadPool::new(),
            object_pool: ObjectPool::new(),
            loggers: RwLock::new(HashMap::new()),
            sinks: RwLock::new(HashMap::new()),
            global_formatter: Arc::new(DefaultFormater::default()),
        }
    }
}

/// 阻塞当前线程，等待 [`AsyncLogger`] 的配套线程完成所有任务并加入。
///
/// [`AsyncLogger`]: crate::logger::async_logger::AsyncLogger
#[inline]
pub fn exit() {
    GLOBAL_DISPATCHER.thread_pool.exit_and_wait();
}

/// 使用注册 [`Sink`] 实例 的名字寻找对应的对象。
///
/// 如果没有注册到 [`GLOBAL_DISPATCHER`] 或者没有该名字，则返回 [`None`]。
#[inline]
pub fn get_sink(name: impl AsRef<str>) -> Option<Arc<dyn Sink>> {
    GLOBAL_DISPATCHER
        .sinks
        .read()
        .unwrap()
        .get(name.as_ref())
        .cloned()
}

/// 使用注册 [`Sink`] 实例 的名字寻找并删除对应的对象。
///
/// 如果没有注册到 [`GLOBAL_DISPATCHER`] 或者没有该名字，则返回 [`None`]。
///
/// 注意，这么做并不会使得该 sink 被禁用，只是将其从 [`GLOBAL_DISPATCHER`] 中移除。
#[inline]
pub fn remove_sink(name: impl AsRef<str>) -> Option<Arc<dyn Sink>> {
    GLOBAL_DISPATCHER
        .sinks
        .write()
        .unwrap()
        .remove(name.as_ref())
}

/// 向 [`GLOBAL_DISPATCHER`] 注册一个 [`Sink`] 对象。
///
/// 如果已有一个同名对象，则将 sink 以 [`Err`] 返回。
#[inline]
pub fn register_sink(sink: Arc<dyn Sink>) -> Result<(), Arc<dyn Sink>> {
    if sink.name() == crate::util::DEFAULT_NULL_NAME {
        return Err(sink);
    }
    match GLOBAL_DISPATCHER
        .sinks
        .write()
        .unwrap()
        .entry(sink.name().into())
    {
        std::collections::hash_map::Entry::Occupied(_) => Err(sink),
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(sink);
            Ok(())
        }
    }
}

/// 使用注册 [`Logger`] 实例 的名字寻找对应的对象。
///
/// 如果没有注册到 [`GLOBAL_DISPATCHER`] 或者没有该名字，则返回 [`None`]。
#[inline]
pub fn get_logger(name: impl AsRef<str>) -> Option<Arc<dyn Logger>> {
    GLOBAL_DISPATCHER
        .loggers
        .read()
        .unwrap()
        .get(name.as_ref())
        .cloned()
}

/// 使用注册 [`Logger`] 实例 的名字寻找并删除对应的对象。
///
/// 如果没有注册到 [`GLOBAL_DISPATCHER`] 或者没有该名字，则返回 [`None`]。
///
/// 注意，这么做并不会使得该 logger 被禁用，只是将其从 [`GLOBAL_DISPATCHER`] 中移除。
#[inline]
pub fn remove_logger(name: impl AsRef<str>) -> Option<Arc<dyn Logger>> {
    GLOBAL_DISPATCHER
        .loggers
        .write()
        .unwrap()
        .remove(name.as_ref())
}

/// 向 [`GLOBAL_DISPATCHER`] 注册一个 [`Sink`] 对象。
///
/// 如果已有一个同名对象，则将 sink 以 [`Err`] 返回。
#[inline]
pub fn register_logger(logger: Arc<dyn Logger>) -> Result<(), Arc<dyn Logger>> {
    match GLOBAL_DISPATCHER
        .loggers
        .write()
        .unwrap()
        .entry(logger.name().into())
    {
        std::collections::hash_map::Entry::Occupied(_) => Err(logger),
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(logger);
            Ok(())
        }
    }
}

/// 获取缓冲池中的一个 [`String`]。
///
/// 所有返回的 [`String`] 都是已清空的。
///
/// 建议在使用完后放回池中。
#[inline]
pub fn acquire_string() -> String {
    GLOBAL_DISPATCHER.object_pool.acquire_string()
}

/// 将一个 [`String`] 放回全局对象池中。
///
/// 如果池已满，将会动态丢弃。
#[inline]
pub fn release_string(mut rel: String) {
    rel.clear();
    GLOBAL_DISPATCHER.object_pool.release_string(rel)
}

/// 设置全局默认的格式化器的格式串。
#[inline]
pub fn set_global_format(pattern: impl AsRef<str>) -> Result<(), super::errors::ParseError> {
    GLOBAL_DISPATCHER
        .global_formatter
        .set_format(pattern.as_ref().to_string())
}

/// 获取全局格式化器实例。
#[inline]
pub fn get_global_formatter() -> Arc<dyn Formatter> {
    GLOBAL_DISPATCHER.global_formatter.clone()
}
