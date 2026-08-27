/// 用于为几个 终端 sik 提供定义。
macro_rules! impl_console_sink {
    ($name:ty, $structor_func:path, $buidler_name:ident) => {
        pub struct $buidler_name {
            name: Box<str>,
            buffer_size: usize,
            formatter: ::std::sync::Arc<dyn Formatter>,
        }

        impl Default for $buidler_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $buidler_name {
            pub fn new() -> Self {
                Self {
                    name: crate::util::DEFAULT_NULL_NAME.into(),
                    buffer_size: 1024 * 16,
                    formatter: $crate::util::dispatcher::get_global_formatter(),
                }
            }

            pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
                self.name = name.as_ref().into();
                self
            }

            pub fn buffer_size(&mut self, size: usize) -> &mut Self {
                self.buffer_size = size;
                self
            }

            pub fn formatter(&mut self, formatter: ::std::sync::Arc<dyn Formatter>) -> &mut Self {
                self.formatter = formatter;
                self
            }

            pub fn build(&self) -> $name {
                <$name>::with_buffer_size_and_formatter(
                    &self.name,
                    self.buffer_size,
                    self.formatter.clone(),
                )
            }
        }

        impl $name {
            pub fn builder() -> $buidler_name {
                <$buidler_name>::default()
            }

            /// see [`BaseSink`](crate::sink::base_sink::BaseSink).
            #[inline]
            fn with_buffer_size_and_formatter(
                name: impl AsRef<str>,
                cap: usize,
                formatter: ::std::sync::Arc<dyn Formatter>,
            ) -> Self {
                Self {
                    inner: $crate::sink::base_sink::BaseSink::with_buffer_size_and_formatter(
                        name,
                        $structor_func(),
                        cap,
                        formatter,
                    ),
                }
            }
        }

        impl Sink for $name {
            fn log(&self, record: &$crate::util::record::Record) -> SinkResult<()> {
                self.inner.log(record)
            }

            fn flush(&self) -> SinkResult<()> {
                self.inner.flush()
            }

            fn name(&self) -> &str {
                self.inner.name()
            }

            fn set_level(&self, level: log::LevelFilter) {
                self.inner.set_level(level);
            }
        }
    };
}
