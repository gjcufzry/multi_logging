//! 写入文件的 [`Sink`]

use std::{fs::File, sync::Mutex};

use crate::{
    fmt::Formatter,
    sink::{Sink, SinkResult, base_sink::BaseSink},
    util::{
        errors::SinkError,
        marker::{AtomicType, NoAtomicType},
    },
};

/// 线程安全的、向一个文件输出的 [`Sink`] 实现。
///
/// # Example
/// ```
/// use multi_logging::sink::FileSinkMT;
/// use multi_logging::fmt::DefaultFormatter;
/// use std::sync::Arc;
///
/// let sink = FileSinkMT::builder()
///     .name("file sink")                                // 名字。
///     .path("app.log")                                  // 打开的文件路径。
///     .buffer_size(1024 * 16)                           // 缓冲区大小。
///     .formatter(Arc::new(DefaultFormatter::default())) // 格式化器。
///     .build();
/// ```
#[repr(transparent)]
pub struct FileSinkMT {
    inner: BaseSink<File, Mutex<()>, AtomicType>,
}

/// 线程不安全的、向一个文件输出的 [`Sink`] 实现。
///
/// # Safety
///
/// - 这个只能在单线程环境使用（或者使用 [`AsyncLogger`] ，并将后台线程池最大线程数量设置为 1 ），
///   任何多线程写入的操作都是未定义行为。
///
/// # Example
/// ```
/// use multi_logging::sink::FileSinkST;
/// use multi_logging::fmt::DefaultFormatter;
/// use std::sync::Arc;
///
/// let sink = FileSinkST::builder()
///     .name("file sink")                                // 名字。
///     .path("app.log")                                  // 打开的文件路径。
///     .buffer_size(1024 * 16)                           // 缓冲区大小。
///     .formatter(Arc::new(DefaultFormatter::default())) // 格式化器。
///     .build();
/// ```
/// [`AsyncLogger`]: crate::logger::AsyncLogger
#[repr(transparent)]
pub struct FileSinkST {
    inner: BaseSink<File, (), NoAtomicType>,
}

macro_rules! impl_file_sink {
    ($name:ty, $builder_name:ident) => {
        impl $name {
            #[inline]
            pub fn builder() -> $builder_name {
                <$builder_name>::new()
            }

            /// see [`BaseSink`](crate::sink::base_sink::BaseSink).
            #[inline]
            fn with_buffer_size_and_formatter(
                name: impl AsRef<str>,
                path: impl AsRef<str>,
                cap: usize,
                formatter: ::std::sync::Arc<dyn Formatter>,
            ) -> SinkResult<Self> {
                Ok(Self {
                    inner: $crate::sink::base_sink::BaseSink::with_buffer_size_and_formatter(
                        name,
                        SinkError::try_with_source(
                            crate::util::errors::SinkErrorKinds::OpenFileError,
                            File::create(path.as_ref()),
                        )?,
                        cap,
                        formatter,
                    ),
                })
            }
        }

        impl Sink for $name {
            #[inline]
            fn log(&self, record: &$crate::util::record::Record) -> SinkResult<()> {
                self.inner.log(record)
            }

            #[inline]
            fn flush(&self) -> SinkResult<()> {
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

        pub struct $builder_name {
            name: Box<str>,
            path: Box<str>,
            formatter: std::sync::Arc<dyn Formatter>,
            buffer_size: usize,
        }

        impl $builder_name {
            #[inline]
            pub fn new() -> Self {
                Self {
                    name: $crate::util::DEFAULT_NULL_NAME.into(),
                    path: "".into(),
                    formatter: $crate::util::dispatcher::get_global_formatter(),
                    buffer_size: super::DEFAULT_BUFFER_SIZE,
                }
            }

            #[inline]
            pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
                self.name = name.as_ref().into();
                self
            }

            #[inline]
            pub fn path(&mut self, path: impl AsRef<str>) -> &mut Self {
                self.path = path.as_ref().into();
                self
            }

            #[inline]
            pub fn buffer_size(&mut self, size: usize) -> &mut Self {
                self.buffer_size = size;
                self
            }

            #[inline]
            pub fn formatter(&mut self, formatter: std::sync::Arc<dyn Formatter>) -> &mut Self {
                self.formatter = formatter;
                self
            }

            #[inline]
            pub fn build(&mut self) -> SinkResult<$name> {
                <$name>::with_buffer_size_and_formatter(
                    &self.name,
                    &self.path,
                    self.buffer_size,
                    self.formatter.clone(),
                )
            }
        }

        impl Default for $builder_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_file_sink!(FileSinkST, FileSinkSTBuilder);
impl_file_sink!(FileSinkMT, FileSinkMTBuilder);
