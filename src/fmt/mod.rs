//! 格式化支持。

pub(crate) mod ansi_color;
pub mod color_console;
pub mod default;
pub(crate) mod dictionary;
pub mod parse;

use crate::util::{Record, errors::ParseError};

pub use color_console::ColorFormatter;
pub use default::DefaultFormatter;
pub use parse::PatternCharacter;

/// 格式化器 api。
pub trait Formatter: Send + Sync {
    /// 设置格式化字符串。
    ///
    /// # Example
    /// ```
    /// use multi_logging::fmt::{DefaultFormatter, Formatter};
    ///
    /// let mut formatter = DefaultFormatter::default();
    /// assert!(formatter.set_format("[%H:%M:%S] [%L]: %v".to_string()).is_ok());
    /// ```
    fn set_format(&self, pattern: String) -> ParseResult<()>;

    /// 格式化字符串。
    ///
    /// # Note
    ///
    /// - 该方法将会被 [`log!`](log::log!) 自动调用。
    fn format(&self, record: &Record) -> String;

    /// 设置不同的日志等级的日志在终端上显示时的颜色。
    ///
    /// # Note
    ///
    /// - 仅支持终端 ANSI 码输入。
    /// # Example
    /// ```
    /// use multi_logging::fmt::{ColorFormatter, Formatter};
    /// use multi_logging::log::Level;
    ///
    /// let formatter = ColorFormatter::new("%^[%Y-%m-%d %H:%M:%S] [%L]: %v%$").unwrap();
    ///
    /// assert!(formatter.set_color(Level::Error, "\x1b[31;1m".to_string()).is_ok());
    /// // 此时，所有使用该 formatter 进行格式化的 sink 且日志级别为 Error 将输出红色加粗的文字（如果输出到终端）。
    fn set_color(&self, level: log::Level, color: String) -> ParseResult<()>;
}

pub type ParseResult<Re> = Result<Re, ParseError>;
