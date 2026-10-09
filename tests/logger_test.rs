use std::sync::Arc;

use multi_logging::logger::Logger;

#[test]
fn test() {
    log::set_max_level(log::LevelFilter::Trace);
    let logger = Arc::new(
        multi_logging::logger::AsyncLogger::builder()
            .add_sink(Arc::new(
                multi_logging::sink::StdoutSinkST::builder()
                    .formatter(Arc::new(
                        multi_logging::fmt::ColorFormatter::new("[%Y-%m-%d %H:%M:%S] [%L] [tid:%t-pid:%P] [%@:%s:%#]: %v").unwrap(),
                    ))
                    .build(),
            ))
            .build(),
    );
    log::info!(logger: *logger, "ciallo");
    logger.flush_and_wait();
}
