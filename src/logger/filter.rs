use crate::{Record, logger::Logger};

type FilterFnTy = Box<dyn Fn(&Record) -> bool + Send + Sync>;

/// 携带过滤函数的 logger。
pub struct FilterLogger<L: Logger> {
    inner: L,
    filter_fn: FilterFnTy,
}

impl<L: Logger> FilterLogger<L> {
    /// 如果回调函数返回true，则继续包装类型的log调用，反之终止调用。
    pub fn new<F: Fn(&Record) -> bool + Send + Sync + 'static>(inner: L, filter: F) -> Self {
        Self {
            inner,
            filter_fn: Box::new(filter),
        }
    }

    /// 解包。
    pub fn unpack(self) -> (L, FilterFnTy) {
        (self.inner, self.filter_fn)
    }

    fn filter(&self, record: &Record) -> bool {
        (self.filter_fn)(record)
    }
}

impl<L: Logger> log::Log for FilterLogger<L> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.log_once(Record::new(record, self.name()));
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

impl<L: Logger> Logger for FilterLogger<L> {
    fn add_sink(&self, sink: std::sync::Arc<dyn crate::sink::Sink>) {
        self.inner.add_sink(sink);
    }

    fn remove_sink(&self, name: String) {
        self.inner.remove_sink(name);
    }

    fn log_once(&self, record: crate::util::Record) {
        if self.filter(&record) {
            self.inner.log_once(record);
        }
    }

    fn flush_and_wait(&self) {
        self.inner.flush_and_wait();
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}
