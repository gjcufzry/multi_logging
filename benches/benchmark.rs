//! 性能测试
//!
//! # Device
//! - cpu: Intel(R) Core(TM) i5-10210U CPU @ 1.60GHz   2.11 GHz
//! - system: Windwos 11 23H2
//! - toolchain: stable-x86_64-pc-windows-msvc 1.96.0
//! - version: 0.1.2
//!
//! # Result
//! ``` bash
//! cargo bench -p multi_logging
//! ```
//! ``` text
//! Running benches\benchmark.rs (target\release\deps\benchmark-47a1a207899a1872.exe)
//! frontend/unbounded_null time:   [29.982 ms 30.081 ms 30.170 ms]
//!                         thrpt:  [3.3145 Melem/s 3.3243 Melem/s 3.3353 Melem/s]
//!                  change:
//!                         time:   [−3.8322% +1.8780% +7.3706%] (p = 0.50 > 0.05)
//!                         thrpt:  [−6.8646% −1.8434% +3.9849%]
//!                         No change in performance detected.
//! Found 5 outliers among 20 measurements (25.00%)
//!   4 (20.00%) low severe
//!   1 (5.00%) high mild
//! frontend/bounded_100k_null
//!                         time:   [23.437 ms 25.521 ms 27.636 ms]
//!                         thrpt:  [3.6185 Melem/s 3.9184 Melem/s 4.2667 Melem/s]
//!                  change:
//!                         time:   [−10.319% −1.2522% +8.5277%] (p = 0.80 > 0.05)
//!                         thrpt:  [−7.8576% +1.2680% +11.506%]
//!                         No change in performance detected.
//! end_to_end/bounded_file_4mb_buffer
//!                         time:   [100.97 ms 101.54 ms 102.14 ms]
//!                         
//!                         thrpt:  [979.02 Kelem/s 984.83 Kelem/s 990.36 Kelem/s]
//!                  change:
//!                         time:   [−2.3909% −0.6307% +0.9131%] (p = 0.49 > 0.05)
//!                         thrpt:  [−0.9049% +0.6347% +2.4495%]
//!                         No change in performance detected.
//! ```
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use multi_logging::fmt::DefaultFormatter;
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
            .formatter(Arc::new(
                DefaultFormatter::new("[%Y-%m-%d %H:%M:%S.%e] [%n] [%l]: %v").unwrap(),
            ))
            .build()
            .unwrap(),
    );
    Arc::new(
        AsyncLogger::builder()
            .name("async_logger")
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
                    // 消息内容与 spdlog 相同
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
