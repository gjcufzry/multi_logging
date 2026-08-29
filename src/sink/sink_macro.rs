/// 用于为几个 终端 sik 提供定义。
macro_rules! impl_console_sink {
    ($name:ty, $structor_func:path, $buidler_name:ident) => {
        #[doc = concat!("[`", stringify!($name), "`] 的构造器。")]
        pub struct $buidler_name {
            name: Box<str>,
            buffer_size: usize,
            formatter: ::std::sync::Arc<dyn Formatter>,
        }

        impl Default for $buidler_name {
            #[inline(always)]
            fn default() -> Self {
                Self::new()
            }
        }

        impl $buidler_name {
            #[inline(always)]
            pub fn new() -> Self {
                Self {
                    name: crate::util::DEFAULT_NULL_NAME.into(),
                    buffer_size: $crate::sink::DEFAULT_BUFFER_SIZE,
                    formatter: $crate::util::dispatcher::get_global_formatter(),
                }
            }

            #[doc = concat!("设置 [`", stringify!($name), "`] 的名字。")]
            #[inline(always)]
            pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
                self.name = name.as_ref().into();
                self
            }

            #[doc = concat!("设置 [`", stringify!($name), "`] 的缓冲区大小。")]
            #[inline(always)]
            pub fn buffer_size(&mut self, size: usize) -> &mut Self {
                self.buffer_size = size;
                self
            }

            #[doc = concat!(
                "设置 [`", stringify!($name), "`] 的格式化器。\n默认为全局格式化器，且开启了终端颜色功能。
            ")]
            #[inline(always)]
            pub fn formatter(&mut self, formatter: ::std::sync::Arc<dyn Formatter>) -> &mut Self {
                self.formatter = formatter;
                self
            }


            #[doc = concat!("最终构造 [`", stringify!($name), "`]。")]
            #[inline(always)]
            pub fn build(&self) -> $name {
                <$name>::with_buffer_size_and_formatter(
                    &self.name,
                    self.buffer_size,
                    self.formatter.clone(),
                )
            }
        }

        impl $name {
            #[doc = concat!("获取 [`", stringify!($name),"`] 对应的 builder。")]
            #[inline(always)]
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
            #[inline(always)]
            fn log(&self, record: &$crate::util::record::Record) -> SinkResult<()> {
                self.inner.log(record)
            }

            #[inline(always)]
            fn flush(&self) -> SinkResult<()> {
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
    };
}
