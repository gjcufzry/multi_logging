//! 格式化支持。

pub mod default;
pub(crate) mod dictionary;
pub mod parse;

use std::sync::{Arc, LazyLock};

use crate::{
    fmt::default::DefaultFormater,
    util::{Record, errors::ParseError},
};

/// 全局默认的格式化器。
pub static GLOBAL_FORMATTER: LazyLock<Arc<dyn Formatter>> =
    LazyLock::new(|| Arc::new(DefaultFormater::default()));

thread_local! {
    /// 线程独立的格式化字符串缓冲区。
    pub static FORMAT_BUFFER: String  = String::with_capacity(1024);
}

/// 格式化器 api。
///
/// 建议使用 [缓冲区](FORMAT_BUFFER) 进行格式化。
pub trait Formatter: Send + Sync {
    /// 设置格式化字符串。
    fn set_format(&self, pattern: String) -> ParseResult<()>;

    /// 格式化字符串。
    fn format(&self, record: &Record) -> String;
}

pub type ParseResult<Re> = Result<Re, ParseError>;
