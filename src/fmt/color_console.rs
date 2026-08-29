use crate::fmt::{
    Formatter, ParseResult, default::DefaultFormatter, dictionary::*, parse::PatternCharacter,
};

use std::{
    fmt::Write,
    time::{Duration, SystemTime},
};

/// 可以使用 ANSI 转义符进行颜色输出的格式化器。
/// 
/// 支持的格式串详见 [`PatternCharacter`]。
///
/// # Example
/// ```
/// use multi_logging::fmt::{ColorFormatter, Formatter};
/// use multi_logging::log::Level;
///
/// let formatter = ColorFormatter::new("%^[%Y-%m-%d %H:%M:%S] [%L]: %v%$").unwrap();
///
/// assert!(formatter.set_color(Level::Error, "\x1b[31;1m".to_string()).is_ok());
/// // 此时，所有使用该 formatter 进行格式化的 sink 且日志级别为 Error 将输出红色加粗的文字（如果输出到终端）。
/// ```
#[derive(Default)]
#[repr(transparent)]
pub struct ColorFormatter {
    pub(crate) inner: DefaultFormatter,
}

include!("./format_macro.rs");

impl Formatter for ColorFormatter {
    #[inline]
    fn set_format(&self, pattern: String) -> ParseResult<()> {
        self.inner.set_format(pattern)
    }

    impl_format!({.inner}, Self::start_color_range, Self::end_color_range);

    #[inline]
    fn set_color(&self, level: log::Level, color: String) -> ParseResult<()> {
        self.inner.set_color(level, color)
    }
}

impl ColorFormatter {
    #[inline]
    pub fn new(pattern: impl AsRef<str>) -> ParseResult<Self> {
        Ok(Self {
            inner: DefaultFormatter::new(pattern)?,
        })
    }

    #[inline(always)]
    fn start_color_range(&self, buf: &mut String, record: &crate::util::Record) {
        let _ = write!(buf, "{}", self.inner.color.get_color(record.level()));
    }

    #[inline(always)]
    fn end_color_range(buf: &mut String) {
        buf.push_str("\x1b[0m");
    }
}

impl From<DefaultFormatter> for ColorFormatter {
    fn from(value: DefaultFormatter) -> Self {
        Self { inner: value }
    }
}
