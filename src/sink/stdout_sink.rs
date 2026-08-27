//! 写入标准输出流的 [`Sink`]

use std::{io::Stdout, sync::Mutex};

use crate::{
    fmt::Formatter,
    sink::{Sink, SinkResult, base_sink::BaseSink},
    util::marker::{AtomicType, NoAtomicType},
};

/// 线程安全的、向标准输出流输出的 [`Sink`] 实现。
#[repr(transparent)]
pub struct StdoutSinkMT {
    inner: BaseSink<Stdout, Mutex<()>, AtomicType>,
}

/// 线程不安全的、向标准输出流输出的 [`Sink`] 实现。
///
/// # Safety
///
/// - 这个只能在单线程环境使用（或者使用 [`AsyncLogger`] ，并将后台线程池最大线程数量设置为 1 ），
///   任何多线程写入的操作都是未定义行为。
///
/// [`AsyncLogger`]: crate::logger::AsyncLogger
#[repr(transparent)]
pub struct StdoutSinkST {
    inner: BaseSink<Stdout, (), NoAtomicType>,
}

include!("./sink_macro.rs");

impl_console_sink!(StdoutSinkMT, std::io::stdout, StdoutSinkMTBuilder);
impl_console_sink!(StdoutSinkST, std::io::stdout, StdoutSinkSTBuilder);
