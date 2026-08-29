use std::fmt::Write;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use crossbeam::atomic::AtomicCell;

use crate::fmt::ansi_color::ANSI;
use crate::{
    fmt::{Formatter, ParseResult, dictionary::PATTERN_CHARS, parse::PatternCharacter},
    util::errors::{ParseError, ParseErrorKind},
};

use super::dictionary::*;

/// 默认的格式串。
///
/// 可以使用 "%+" 格式化参数, 同样也可以使用 [`Default`] 提供的方法构造.
/// 格式化结果是这样:
/// ``` text
/// [2026-08-27 09:39:34] [INFO]: some message.
/// ```
pub const DEFAULT_PATTERN_STRING: &str = "[%Y-%m-%d %H:%M:%S] [%L]: %v";

/// 保存不同 [`Level`](log::Level) 对应的终端颜色。
pub(crate) struct LevelColor {
    inner: [RwLock<ANSI>; 5], // 按照等级排序。
}

impl LevelColor {
    #[inline(always)]
    pub(crate) fn set_color(
        &self,
        level: log::Level,
        color: impl AsRef<str>,
    ) -> Result<(), ParseError> {
        let ansi = color.as_ref().parse()?;
        *self.inner[level as usize - 1].write().unwrap() = ansi;
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn get_color(&self, level: log::Level) -> ANSI {
        self.inner[level as usize - 1].read().unwrap().clone()
    }
}

impl Default for LevelColor {
    fn default() -> Self {
        let mut iter = DEFAULT_LOG_LEVEL_COLOR
            .iter()
            .map(|color| RwLock::new(color.parse::<ANSI>().unwrap()));
        Self {
            // 一定有 5 个。
            inner: [
                iter.next().unwrap(),
                iter.next().unwrap(),
                iter.next().unwrap(),
                iter.next().unwrap(),
                iter.next().unwrap(),
            ],
        }
    }
}

/// 默认的格式化器，不直接支持终端颜色显示。
/// 
/// 支持的格式串详见 [`PatternCharacter`]。
/// 
/// # Example
/// ```
/// use multi_logging::fmt::DefaultFormatter;
/// 
/// let formatter = DefaultFormatter::new("[%Y-%m-%d %H:%M:%S] [%L]: %v");
/// ```
pub struct DefaultFormatter {
    pub(crate) pattern: RwLock<Vec<PatternCharacter>>,
    pub(crate) last_parse: AtomicCell<SystemTime>,
    pub(crate) color: LevelColor,
}

include!("./format_macro.rs");

impl Formatter for DefaultFormatter {
    #[inline]
    fn set_format(&self, pattern: String) -> super::ParseResult<()> {
        *self.pattern.write().unwrap() = Self::preprocess(pattern)?;
        Ok(())
    }

    impl_format!({}, null_1, null_2);

    #[inline]
    fn set_color(&self, level: log::Level, color: String) -> ParseResult<()> {
        self.color.set_color(level, color)
    }
}

impl DefaultFormatter {
    #[inline]
    pub fn new(pattern: impl AsRef<str>) -> ParseResult<Self> {
        Ok(Self {
            pattern: RwLock::new(Self::preprocess(pattern)?),
            last_parse: AtomicCell::new(SystemTime::now()),
            color: LevelColor::default(),
        })
    }

    #[inline(always)]
    fn preprocess(pattern: impl AsRef<str>) -> ParseResult<Vec<PatternCharacter>> {
        let mut res = Vec::with_capacity(pattern.as_ref().len());
        let mut iter = pattern.as_ref().chars().enumerate();

        while let Some((_, ch)) = iter.next() {
            if ch == '%'
                && let Some((idx_ne, ch_ne)) = iter.next()
            {
                let Some(&pattern) =
                    PATTERN_CHARS.get((ch_ne as usize).wrapping_sub(b' ' as usize))
                else {
                    // 不在目标范围中。
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedPattern(ch_ne),
                        idx_ne,
                    ));
                };
                if pattern == PatternCharacter::Void {
                    // 范围中的无效字符。
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedPattern(ch_ne),
                        idx_ne,
                    ));
                } else if pattern == PatternCharacter::DefaultFormat {
                    res.extend(Self::preprocess(DEFAULT_PATTERN_STRING)?);
                }
                res.push(pattern);
                continue;
            }
            res.push(PatternCharacter::Literal(ch));
        }

        Ok(res)
    }
}

impl Default for DefaultFormatter {
    fn default() -> Self {
        Self::new(DEFAULT_PATTERN_STRING).unwrap()
    }
}

#[inline(always)]
fn null_1(_: &DefaultFormatter, _: &mut String, _: &crate::util::Record) {}

#[inline(always)]
fn null_2(_: &mut String) {}

impl From<super::ColorFormatter> for DefaultFormatter {
    fn from(value: super::ColorFormatter) -> Self {
        value.inner
    }
}
