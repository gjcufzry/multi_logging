use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub struct BaseError<Kind: Debug + Display> {
    pub(crate) kind: Kind,
    pub(crate) source: Option<Box<dyn Error + 'static>>,
}

impl<Kind: Debug + Display> Error for BaseError<Kind> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref()
    }
}

impl<Kind: Debug + Display> BaseError<Kind> {
    pub fn new(kind: Kind) -> Self {
        Self { kind, source: None }
    }

    pub fn with_source(kind: Kind, source: impl Error + 'static) -> Self {
        Self {
            kind,
            source: Some(Box::new(source)),
        }
    }

    /// 如果 raw_error 是 Err(_)，则构造一个被  Err(_) 包裹的 [`BaseError`]，并且记录为 source
    #[inline]
    pub fn try_with_source<T1, T2: Error + 'static>(
        kind: Kind,
        raw_error: Result<T1, T2>,
    ) -> Result<T1, Self> {
        match raw_error {
            Ok(ok_val) => Ok(ok_val),
            Err(err_val) => Err(Self::with_source(kind, err_val)),
        }
    }
}

impl<Kind: Debug + Display> Display for BaseError<Kind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.kind))
    }
}

pub type SinkError = BaseError<SinkErrorKinds>;

#[derive(Debug, Clone, Copy)]
pub enum SinkErrorKinds {
    FlushError,
    WriteError,
    OpenFileError,
    DuplicateName,
}

impl Display for SinkErrorKinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str({
            match self {
                Self::FlushError => "sink error when flushing.",
                Self::WriteError => "Sink error when writing to the output.",
                Self::OpenFileError => "Could not open file.",
                Self::DuplicateName => "Crate sink with a duplicate name.",
            }
        })
    }
}

pub type LoggerError = BaseError<LoggerErrorKind>;

#[derive(Debug)]
pub enum LoggerErrorKind {
    SinkNameNotFound,
    FlushError,
    LogError,
}

impl Display for LoggerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str({
            match self {
                LoggerErrorKind::SinkNameNotFound => {
                    "Can't find the name on the logger's sink list."
                }
                LoggerErrorKind::FlushError => "Error when flushing.",
                LoggerErrorKind::LogError => "Error when loging",
            }
        })
    }
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    offset: usize,
}

impl ParseError {
    pub const fn new(kind: ParseErrorKind, offset: usize) -> Self {
        Self { kind, offset }
    }

    #[inline]
    fn as_string(&self) -> String {
        match self.kind {
            ParseErrorKind::UnexpectedPattern(c) => {
                format!(
                    "Unexpected pattern character. Found '{c}' at offset {}.",
                    self.offset
                )
            }
            ParseErrorKind::PatternNotFound => {
                "Expected a pattern character, but the pattern string ended.".to_string()
            }
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedPattern(char),
    PatternNotFound,
}
