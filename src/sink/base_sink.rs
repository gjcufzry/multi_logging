//! 基础 [`Sink`] 实现。

use std::{
    cell::UnsafeCell,
    io::{BufWriter, Write},
    sync::Arc,
};

use crate::{
    fmt::Formatter,
    sink::{Sink, SinkResult},
    util::{
        Record,
        errors::{SinkError, SinkErrorKinds},
        marker::{MaybeAtomicOperation, MaybeAtomicType, MaybeMutexType, NoAtomicType},
    },
};

/// 一个基础的、带有缓冲区的 [`Sink`] 实现。
///
/// 只有最基础的功能，需要写入对象实现了 [`Write`] trait。
///
/// `Output` 泛型是日志输出的地方。
/// `Mutex` 泛型在这里只是作为同步手段存在，默认值为 `()`，即不同步。
pub struct BaseSink<Output, Mutex = (), Atomic = NoAtomicType>
where
    Output: Write,
    Mutex: MaybeMutexType,
    Atomic: MaybeAtomicType<log::LevelFilter>,
{
    pub(crate) name: Box<str>,
    pub(crate) filter: Atomic::Inner<log::LevelFilter>,
    pub(crate) mutex: Mutex, // 可能存在的锁。
    pub(crate) inner: UnsafeCell<BufWriter<Output>>,
    pub(crate) formatter: Arc<dyn Formatter>,
}

// SAFETY:
//
// - 这部分应当由实现来保证。
unsafe impl<Output, Mutex, Atomic> Sync for BaseSink<Output, Mutex, Atomic>
where
    Output: Write,
    Mutex: MaybeMutexType<Inner = ()>,
    Atomic: MaybeAtomicType<log::LevelFilter>,
{
}

// SAFETY:
//
// - 这部分应当由实现来保证。
unsafe impl<Output, Mutex, Atomic> Send for BaseSink<Output, Mutex, Atomic>
where
    Output: Write,
    Mutex: MaybeMutexType<Inner = ()>,
    Atomic: MaybeAtomicType<log::LevelFilter>,
{
}

impl<Output, Mutex, Atomic> Sink for BaseSink<Output, Mutex, Atomic>
where
    Output: Write,
    Mutex: MaybeMutexType<Inner = ()>,
    Atomic: MaybeAtomicType<log::LevelFilter>,
{
    #[inline]
    fn log(&self, record: &Record) -> SinkResult<()> {
        if self.filter.may_atomic_load() < record.level() {
            return Ok(());
        }

        let format_res = self.formatter.format(record);
        let _mutex = self.mutex.may_mutex_lock();
        // SAFETY: 已经获取了（可能存在的）锁，根据使用场景不同，这么做已经安全了。
        let res = SinkError::try_with_source(SinkErrorKinds::WriteError, unsafe {
            writeln!(*self.inner.get(), "{}", format_res)
        });
        crate::util::dispatcher::release_string(format_res);
        res
    }

    #[inline]
    fn flush(&self) -> SinkResult<()> {
        let _mutex = self.mutex.may_mutex_lock();
        // SAFETY: 已经获取了（可能存在的）锁，根据使用场景不同，这么做已经安全了。
        SinkError::try_with_source(SinkErrorKinds::FlushError, unsafe {
            (*self.inner.get()).flush()
        })
    }

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn set_level(&self, level: log::LevelFilter) {
        self.filter.may_atomic_store(level);
    }
}

impl<Output, Mutex, Atomic> BaseSink<Output, Mutex, Atomic>
where
    Output: Write,
    Mutex: MaybeMutexType<Inner = ()>,
    Atomic: MaybeAtomicType<log::LevelFilter>,
{
    /// 同时指定缓冲区大小与格式化器创建。
    #[inline]
    pub fn with_buffer_size_and_formatter(
        name: impl AsRef<str>,
        output: Output,
        cap: usize,
        formatter: Arc<dyn Formatter>,
    ) -> Self {
        Self {
            name: Box::from(name.as_ref()),
            filter: MaybeAtomicOperation::may_atomic_new(log::LevelFilter::Trace),
            mutex: MaybeMutexType::may_mutex_new(()),
            inner: UnsafeCell::new(BufWriter::with_capacity(cap, output)),
            formatter,
        }
    }
}
