//! 写入标准错误流的 [`Sink`]

use std::{io::Stderr, sync::Mutex};

use crate::{
    fmt::Formatter,
    sink::{Sink, SinkResult, base_sink::BaseSink},
    util::marker::{AtomicType, NoAtomicType},
};

/// 线程安全的、向标准错误流输出的 [`Sink`] 实现。
///
/// # Example
/// ```
/// use multi_logging::sink::StderrSinkMT;
///
/// let sink = StderrSinkMT::builder()
///     .name("stderr sink")
///     .buffer_size(1024 * 16)
///     .build();
/// ```
#[repr(transparent)]
pub struct StderrSinkMT {
    inner: BaseSink<Stderr, Mutex<()>, AtomicType>,
}

/// 线程不安全的、向标准错误流输出的 [`Sink`] 实现。
///
/// # Example
/// ```
/// use multi_logging::sink::StderrSinkST;
///
/// let sink = StderrSinkST::builder()
///     .name("stderr sink")
///     .buffer_size(1024 * 16)
///     .build();
/// ```
///
/// # Safety
///
/// - 这个只能在单线程环境使用（或者使用 [`AsyncLogger`] ，并将后台线程池最大线程数量设置为 1 ），
///   任何多线程写入的操作都是未定义行为。
///
/// [`AsyncLogger`]: crate::logger::AsyncLogger
#[repr(transparent)]
pub struct StderrSinkST {
    inner: BaseSink<Stderr, (), NoAtomicType>,
}

include!("./sink_macro.rs");

impl_console_sink!(StderrSinkST, std::io::stderr, StderrSinkSTBuilder);
impl_console_sink!(StderrSinkMT, std::io::stderr, StderrSinkMTBuilder);
