//! 文本解析。

/// 表示跟在 `%` 后面的单个模式字符的含义。
/// 
/// # Detail
/// 
/// | 占位符 | 说明 |
/// |---|---|
/// | `%v` | 实际的日志消息文本 |
/// | `%n` | logger 名 |
/// | `%l` | 小写日志级别 |
/// | `%L` | 大写日志级别 |
/// | `%t` | 线程 ID |
/// | `%P` | 进程 ID |
/// | `%@` | 源文件位置 |
/// | `%s` | 源文件名 |
/// | `%g` | 源文件的短文件名 |
/// | `%#` | 源码行号 |
/// | `%!` | 源码函数名 |
/// | `%Y` | 四位数年份 |
/// | `%y` | 两位数年份 |
/// | `%m` | 月份 |
/// | `%d` | 天数 |
/// | `%H` | 24 小时制的小时 |
/// | `%I` | 12 小时制的小时 |
/// | `%M` | 分钟 |
/// | `%S` | 秒数 |
/// | `%e` | 毫秒 |
/// | `%f` | 微秒 |
/// | `%F` | 纳秒 |
/// | `%p` | AM/PM 标识 |
/// | `%z` | UTC 时区偏移量 |
/// | `%E` | UTC 时间戳 |
/// | `%c` | 标准日期时间 |
/// | `%i` | 距上条日志的微秒间隔 |
/// | `%u` | 距上条日志的纳秒间隔 |
/// | `%^` / `%$` | 颜色范围起止 |
/// | `%%` | 字面量 `%` |
/// | `%&` | 所有 MDC 键值对（尚未实现，目前仅占位） |
/// | `%+` | 默认格式 |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCharacter {
    /// 无特殊作用，解析到这个会报错。
    Void,
    /// %v  实际的日志消息文本。
    Message,
    /// %n  logger 名。
    LoggerName,
    /// %l  小写日志级别。
    LevelLower,
    /// %L  大写日志级别。
    LevelUpper,
    /// %t  线程ID。
    ThreadId,
    /// %P  进程ID。
    ProcessId,
    /// %@  源文件位置。
    SourceLocation,
    /// %s  源文件名。
    SourceFile,
    /// %g  源文件的短文件名。
    SourceShortFile,
    /// %#  源码行号。
    Line,
    /// %!  源码函数名。
    FuncName,
    /// %Y  四位数年份。
    Year4,
    /// %y  两位数年份。
    Year2,
    /// %m  月份。
    Month,
    /// %d  天数。
    Day,
    /// %H  24小时制的小时。
    Hour24,
    /// %I  12小时制的小时。
    Hour12,
    /// %M  分钟。
    Minute,
    /// %S  秒数。
    Second,
    /// %e  毫秒。
    Millisecond,
    /// %f  微秒。
    Microsecond,
    /// %F  纳秒。
    Nanosecond,
    /// %p  AM/PM 标识。
    AMPM,
    /// %z  UTC 时区偏移量。
    TimezoneOffset,
    /// %E  UTC 时间戳。
    UnixTimestamp,
    /// %c  标准日期时间。
    StandardDateTime,
    /// %i  距上条日志的微秒间隔。
    ElapsedMicroseconds,
    /// %u  距上条日志的纳秒间隔。
    ElapsedNanoseconds,
    /// %^  颜色范围开始。
    StartColorRange,
    /// %$  颜色范围结束。
    StopColorRange,
    /// %%  字面量。
    Literal(char),
    /// %&  所有 MDC 键值对。
    /// TODO: 该方法现在还没有实现。
    AllMappedDiagnosticContext,
    /// %+  默认格式。
    DefaultFormat,
}
