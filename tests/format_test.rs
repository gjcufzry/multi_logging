//! 测试格式化。
//! 
//! # 测试命令
//! 
//! ``` text
//! cargo test -p multi_logging --lib -F kv -- --nocapture --test-threads=1
//! cargo test -p multi_logging --lib -- --nocapture --test-threads=1
//! ```

use multi_logging::{
    fmt::Formatter, global::{get_global_formatter, set_global_format}, log::*, logger::SyncLogger, sink::{Sink, SinkResult},
};
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct StringSink {
    inner: Mutex<String>,
}

impl Sink for StringSink {
    /// 修改为清空。
    fn flush(&self) -> SinkResult<()> {
        self.inner.lock().unwrap().clear();
        Ok(())
    }

    fn name(&self) -> &str {
        "Hello World"
    }

    fn set_level(&self, _level: log::LevelFilter) {}
    
    fn log(&self, record: &multi_logging::Record) -> SinkResult<()> {
        write!(self.inner.lock().unwrap(), "{}", get_global_formatter().format(record, "Hello World")).unwrap();
        Ok(())
    }
}

impl StringSink {
    fn get_inner(&self) -> Box<str> {
        self.inner.lock().unwrap().as_str().into()
    }
}

#[test]
fn format_test() {
    let sink = Arc::new(StringSink::default());
    let logger = SyncLogger::builder()
        .name("sync_logger")
        .level(LevelFilter::Info)
        .add_sink(sink.clone())
        .build();

    set_max_level(LevelFilter::Trace);

    assert!(set_global_format("").is_ok());
    info!(logger: logger, "Hello World");
    assert_eq!(sink.get_inner(), "".into());
    logger.flush();

    assert!(set_global_format("%% ciallo!").is_ok());
    info!(logger: logger, "Hello World");
    assert_eq!(sink.get_inner(), "% ciallo!".into());
    logger.flush();

    assert!(set_global_format("[%l] [%L] [%n]: %v").is_ok());
    info!(logger: logger, "Hello World");
    assert_eq!(
        sink.get_inner(),
        "[info] [INFO] [sync_logger]: Hello World".into()
    );
    logger.flush();
    warn!(logger: logger, "Hello World");
    assert_eq!(
        sink.get_inner(),
        "[warn] [WARN] [sync_logger]: Hello World".into()
    );
    logger.flush();

    set_max_level(LevelFilter::Warn);

    assert!(set_global_format("[%l] [%L] [%n]: %v").is_ok());
    info!(logger: logger, "Hello World");
    assert_eq!(sink.get_inner(), "".into());
    logger.flush();
    warn!(logger: logger, "Hello World");
    assert_eq!(
        sink.get_inner(),
        "[warn] [WARN] [sync_logger]: Hello World".into()
    );
    logger.flush();
}

#[cfg(feature = "kv")]
#[test]
fn kv_test() {
    #[allow(unused)]
    #[derive(Debug)]
    struct Foo {
        x: usize,
        y: &'static str,
    }

    let sink = Arc::new(StringSink::default());
    let logger = SyncLogger::builder()
        .name("sync_logger")
        .level(LevelFilter::Info)
        .add_sink(sink.clone())
        .build();

    set_max_level(LevelFilter::Info);

    let test_tuple = (
        "Hello World",
        114514,
        false,
        Foo {
            x: 12,
            y: "ciallo!",
        },
    );

    assert!(set_global_format("{%v} kv: %&").is_ok());

    info!(logger: logger, test_tuple:?, ciallo="ciallo"; "Test Kv.");
    assert_eq!(
        sink.get_inner(),
        "{Test Kv.} kv: test_tuple=(\"Hello World\", 114514, false, Foo { x: 12, y: \"ciallo!\" }) ciallo=ciallo".into()
    );
    logger.flush();
}
