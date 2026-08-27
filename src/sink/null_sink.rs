use crate::sink::Sink;

/// 一个没有任何输出的 [`Sink`] 实现。
pub struct NullSink {
    name: Box<str>,
}

impl Sink for NullSink {
    fn log(&self, _record: &crate::util::Record) -> super::SinkResult<()> {
        Ok(())
    }

    fn flush(&self) -> super::SinkResult<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_level(&self, _level: log::LevelFilter) {}
}

impl NullSink {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().into(),
        }
    }

    pub fn builder() -> NullSinkBuilder {
        NullSinkBuilder::default()
    }
}

pub struct NullSinkBuilder {
    name: Box<str>,
}

impl NullSinkBuilder {
    pub fn new() -> Self {
        Self {
            name: crate::util::DEFAULT_NULL_NAME.into(),
        }
    }

    pub fn name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.name = name.as_ref().into();
        self
    }

    pub fn build(&self) -> NullSink {
        NullSink::new(self.name.clone())
    }
}

impl Default for NullSinkBuilder {
    fn default() -> Self {
        Self::new()
    }
}
