use std::sync::RwLock;
use std::{fmt::Write, time::Instant};

use crossbeam::atomic::AtomicCell;

use crate::{
    fmt::{Formatter, ParseResult, dictionary::PATTERN_CHARS, parse::PatternCharacter},
    util::errors::{ParseError, ParseErrorKind},
};

use super::dictionary::*;

/// 默认的格式串。
///
/// 可以使用 "%+" 格式化参数, 同样也可以使用 [`Default`] 提供的方法构造.
/// 格式化结果是这样:
/// ``` ignore
/// [2026-08-27 09:39:34] [INFO]: some message.
/// ```
pub const DEFAULT_PATTERN_STRING: &str = "[%Y-%m-%d %H:%M:%S] [%L]: %v";

/// 小写形式的 log::Level 字符串显示。
const LOG_LEVEL_NAMES_LOWER: [&str; 6] = ["off", "error", "warn", "info", "debug", "trace"];

/// 默认的格式化器。
pub struct DefaultFormater {
    pattern: RwLock<Vec<PatternCharacter>>,
    last_parse: AtomicCell<Instant>,
}

impl Formatter for DefaultFormater {
    fn set_format(&self, pattern: String) -> super::ParseResult<()> {
        *self.pattern.write().unwrap() = Self::preprocess(pattern)?;
        Ok(())
    }

    fn format(&self, record: &crate::util::Record) -> String {
        let mut buf = crate::util::dispatcher::acquire_string();
        let time =
            time::OffsetDateTime::from_unix_timestamp_nanos(record.time_stamp_nano() as i128)
                .unwrap()
                .to_offset(time::UtcOffset::current_local_offset().unwrap());

        for pattern in self.pattern.read().unwrap().iter() {
            match pattern {
                PatternCharacter::Void => unreachable!(), // 已经在初始化时排除。
                PatternCharacter::Message => {
                    let _ = write!(buf, "{}", record.log_detail());
                }
                PatternCharacter::LoggerName => {
                    buf.push_str(record.logger_name());
                }
                PatternCharacter::LevelLower => {
                    buf.push_str(LOG_LEVEL_NAMES_LOWER[record.level() as usize]);
                }
                PatternCharacter::LevelUpper => {
                    buf.push_str(record.level().as_str());
                }
                PatternCharacter::ThreadId => {
                    let _ = write!(buf, "{:?}", record.thread_id());
                }
                PatternCharacter::ProcessId => {
                    let _ = write!(buf, "{}", record.process_id());
                }
                PatternCharacter::SourceLocation => {
                    buf.push_str(record.module_path().unwrap_or("?"));
                }
                PatternCharacter::SourceFile => {
                    buf.push_str(record.file().unwrap_or("?"));
                }
                PatternCharacter::SourceShortFile => {
                    buf.push_str(record.file().unwrap_or("?"));
                }
                PatternCharacter::Line => {
                    if let Some(line) = record.line() {
                        let _ = write!(buf, "{}", line);
                    } else {
                        buf.push('?');
                    }
                }
                PatternCharacter::FuncName => unimplemented!(),
                PatternCharacter::Year4 => {
                    let _ = write!(buf, "{}", YEAR4[time.year() as usize - 1970]);
                }
                PatternCharacter::Year2 => {
                    let _ = write!(buf, "{}", PAD2[(time.year() % 100) as usize]);
                }
                PatternCharacter::Month => {
                    let _ = write!(buf, "{}", PAD2[time.month() as usize]);
                }
                PatternCharacter::Day => {
                    let _ = write!(buf, "{}", PAD2[time.day() as usize]);
                }
                PatternCharacter::Hour24 => {
                    let _ = write!(buf, "{}", PAD2[time.hour() as usize]);
                }
                PatternCharacter::Hour12 => {
                    let _ = write!(buf, "{}", PAD2[time.hour().saturating_sub(12) as usize]);
                }
                PatternCharacter::Minute => {
                    let _ = write!(buf, "{}", PAD2[time.minute() as usize]);
                }
                PatternCharacter::Second => {
                    let _ = write!(buf, "{}", PAD2[time.second() as usize]);
                }
                PatternCharacter::Millisecond => {
                    let _ = write!(buf, "{}", PAD3[time.millisecond() as usize]);
                }
                PatternCharacter::Microsecond => {
                    let _ = write!(buf, "{}", PAD3[time.microsecond() as usize % 1000]);
                }
                PatternCharacter::Nanosecond => {
                    let _ = write!(buf, "{}", PAD3[(time.nanosecond() % 1000) as usize]);
                }
                PatternCharacter::AMPM => {
                    if time.hour() >= 12 {
                        let _ = write!(buf, "PM");
                    } else {
                        let _ = write!(buf, "AM");
                    }
                }
                PatternCharacter::TimezoneOffset => {
                    let _ = write!(buf, "{}", time.offset());
                }
                PatternCharacter::UnixTimestamp => {
                    let _ = write!(buf, "{}", time.unix_timestamp());
                }
                PatternCharacter::StandardDateTime => unreachable!(), // 在初始化阶段就被替换。
                PatternCharacter::ElapsedMicroseconds => {
                    let _ = write!(buf, "{:06}", self.last_parse.load().elapsed().as_micros());
                }
                PatternCharacter::ElapsedNanoseconds => {
                    let _ = write!(buf, "{:09}", self.last_parse.load().elapsed().as_nanos());
                }
                PatternCharacter::StartColorRange => {
                    buf.push_str("\x1b[1;31m");
                }
                PatternCharacter::StopColorRange => {
                    buf.push_str("\x1b[0m");
                }
                PatternCharacter::Literal(c) => {
                    buf.push(*c);
                }
                PatternCharacter::AllMappedDiagnosticContext => {
                    unimplemented!("将会围绕 log 库的 kv 模块实现。")
                }
                PatternCharacter::DefaultFormat => unreachable!(), // 在初始化阶段就被替换。
            }
        }

        buf
    }
}

impl DefaultFormater {
    pub fn new(pattern: impl AsRef<str>) -> ParseResult<Self> {
        Ok(Self {
            pattern: RwLock::new(Self::preprocess(pattern)?),
            last_parse: AtomicCell::new(Instant::now()),
        })
    }

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

impl Default for DefaultFormater {
    fn default() -> Self {
        Self::new(DEFAULT_PATTERN_STRING).unwrap()
    }
}
