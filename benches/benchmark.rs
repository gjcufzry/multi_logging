use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use multi_logging::logger::Logger;
use multi_logging::logger::async_logger::AsyncLogger;
use multi_logging::sink::file_sink::FileSinkST;
use multi_logging::sink::null_sink::NullSink;
use std::sync::Arc;
use std::time::Duration;

/// 创建使用 NullSink 的异步 logger，用于纯前端测试
fn create_null_logger(bounded: bool, queue_size: usize) -> Arc<AsyncLogger> {
    let sink = Arc::new(NullSink::new("null"));
    Arc::new(
        AsyncLogger::builder()
            .add_sink(sink)
            .bound(bounded)
            .drop_when_blocked(false) // 对于有界队列使用 block 策略
            .chanel_size(queue_size)
            .build(),
    )
}

/// 创建使用文件 sink 的异步 logger，用于端到端测试
fn create_file_logger(bounded: bool, queue_size: usize, buffer_size: usize) -> Arc<AsyncLogger> {
    let sink = Arc::new(
        FileSinkST::builder()
            .name("file_sink")
            .path("bench.log")
            .buffer_size(buffer_size)
            .build()
            .unwrap(),
    );
    Arc::new(
        AsyncLogger::builder()
            .add_sink(sink)
            .bound(bounded)
            .drop_when_blocked(false)
            .chanel_size(queue_size)
            .build(),
    )
}

/// 执行多线程日志发送，可选择是否等待 flush
fn run_multi_thread_bench(
    logger: Arc<AsyncLogger>,
    threads: usize,
    per_thread: usize,
    wait_flush: bool,
) {
    std::thread::scope(|s| {
        for _ in 0..threads {
            let l = logger.clone();
            s.spawn(move || {
                for i in 0..per_thread {
                    // 使用 black_box 防止优化，同时消息内容与 spdlog 相同
                    log::info!(
                        logger: *l,
                        "Hello logger: msg number {}",
                        i
                    );
                }
            });
        }
    });

    if wait_flush {
        logger.flush_and_wait(); // 等待所有日志处理完毕并刷盘
    }
}

/// 基准组1：纯前端发送性能（无 flush，NullSink）
fn bench_frontend(c: &mut Criterion) {
    log::set_max_level(log::LevelFilter::Trace);
    let mut group = c.benchmark_group("frontend");
    group.throughput(Throughput::Elements(100_000)); // 每次迭代处理 100k 条

    // 无界队列
    group.bench_function("unbounded_null", |b| {
        let logger = create_null_logger(false, 0);
        b.iter(|| run_multi_thread_bench(logger.clone(), 10, 10_000, false));
    });

    // 有界队列，容量等于总消息数
    group.bench_function("bounded_100k_null", |b| {
        let logger = create_null_logger(true, 100_000);
        b.iter(|| run_multi_thread_bench(logger.clone(), 10, 10_000, false));
    });

    group.finish();
}

/// 基准组2：端到端性能（文件 sink，包含 flush）
fn bench_end_to_end(c: &mut Criterion) {
    log::set_max_level(log::LevelFilter::Trace);
    let mut group = c.benchmark_group("end_to_end");
    group.throughput(Throughput::Elements(100_000));

    // 有界队列，大缓冲区
    group.bench_function("bounded_file_4mb_buffer", |b| {
        let logger = create_file_logger(true, 100_000, 4 * 1024 * 1024);
        b.iter(|| run_multi_thread_bench(logger.clone(), 10, 10_000, true));
    });

    // 也可以增加无界队列的情况，但无界+flush 可能导致内存爆炸，不推荐
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(20)              // 每个基准采样 20 次，减少总时间
        .measurement_time(Duration::from_secs(10)); // 测量时间
    targets = bench_frontend, bench_end_to_end
);
criterion_main!(benches);
