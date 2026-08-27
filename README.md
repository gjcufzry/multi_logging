# multi_logging

[![GitHub](https://img.shields.io/badge/GitHub-Repository-blue?logo=github)](https://github.com/gjcufzry/multi_logging)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个高性能的 Rust 异步日志库，提供灵活的格式化、多 sink 支持和优秀的并发性能。它实现了 [`log`] crate 的 `Log` trait，可作为 [`log`] 的后端。

## ✨ 特性

- **异步日志**：后台线程处理日志写入，前端调用仅将日志推入无锁队列，不阻塞业务线程。
- **可定制格式**：支持类似 C 风格 `printf` 的日志格式字符串，内置常用占位符（时间、级别、线程、文件位置等）。
- **多种 Sink**：
  - 文件输出 ([`FileSinkMT`] / [`FileSinkST`])
  - 标准输出/错误 ([`StdoutSinkMT`] / [`StderrSinkMT`] 等)
  - 空输出 ([`NullSink`]，用于测试或禁用日志)
  - 钩子 ([`HookSink`]，在日志前后执行自定义回调)
- **高效设计**：
  - 对象池复用字符串缓冲区，减少内存分配。
  - 预计算时间数字表，加速时间戳格式化。
  - 基于 [`crossbeam`] 的高性能无锁队列。
  - 可配置队列大小及溢出策略（阻塞或丢弃）。
- **易于集成**：直接作为 [`log`] 的后端，与现有 Rust 日志生态无缝衔接。

## 📦 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
multi_logging = "0.1"
log = "0.4"
```
- `log` 依赖是可选的，因为其已经在 crate::log 重新导出。

## 🚀 快速开始

### 使用全局 logger

可以直接将 logger 注册为 [`log`] 中的全局对象。

```rust
use multi_logging::log;
use multi_logging::logger::{AsyncLogger, Logger};
use multi_logging::sink::FileSinkMT;
use std::sync::Arc;

fn main() {
    // 1. 设置全局日志级别
    log::set_max_level(log::LevelFilter::Trace);

    // 2. 创建 sink
    let sink = Arc::new(
        FileSinkMT::builder()
            .path("app.log")
            .buffer_size(1024 * 1024) // 1MB 缓冲区
            .build()
            .expect("创建 sink 失败"),
    );

    // 3. 创建 logger
    let logger = Arc::new(
        AsyncLogger::builder()
            .name("main")
            .add_sink(sink)
            .bound(true)               // 有界队列
            .chanel_size(100_000)      // 队列容量
            .drop_when_blocked(false)  // 队列满时阻塞而非丢弃
            .build(),
    );

    // 4. 记录日志
    log::info!(logger: *logger, "Hello, world!");
    log::info!(logger: *logger, "Number: {}", 42);

    // 5. 可以将 logger 注册为 log crate 的全局 logger
    let _ = log::set_boxed_logger(Box::new(logger.clone()));

    log::info!("The global logger send.");

    // 6.如果使用异步 logger，最好在程序结束时显式刷新并等待日志落盘。
    logger.flush_and_wait();
}
```

### 使用注册 API

你也可以通过 [`register_logger`] 和 [`register_sink`] 进行全局注册。
注册之后可以通过 logger 或 sink 的名字获取对应实例。

```rust
use multi_logging::logger::{AsyncLogger, Logger};
use multi_logging::sink::FileSinkMT;
use multi_logging::{log, get_logger, register_logger, register_sink, set_global_format};
use std::sync::Arc;

fn main() {
    // 设置全局过滤等级
    log::set_max_level(log::LevelFilter::Trace);

    // 注册 sink
    let sink = Arc::new(
        FileSinkMT::builder()
            .name("file")
            .path("log.log")
            .build()
            .unwrap(),
    );
    let _ = register_sink(sink.clone());

    // 注册 logger 并关联 sink
    let logger = Arc::new(AsyncLogger::builder().name("global").add_sink(sink).build());
    let _ = register_logger(logger.clone());

    // 设置全局格式
    let _ = set_global_format("[%Y-%m-%d %H:%M:%S] [%L] %v");

    // 使用 log 宏即可
    log::info!(logger: *logger, "This goes to the file");

    // 获取注册的 logger 的相同实例
    let same_logger = get_logger("global").unwrap();

    // 可以正常发送日志。
    log::info!(logger: *same_logger, "The same_logger send to the same file.");

    // 显式刷新缓冲。
    logger.flush_and_wait();
}
```

## 📝 日志格式

默认格式：`"[%Y-%m-%d %H:%M:%S] [%L]: %v"`。  
可通过 [`set_global_format`] 或大部分的 sink builer 提供的 `formatter` 方法自定义。支持的占位符：

| 占位符 | 说明               |
|--------|--------------------|
| `%v`   | 日志消息           |
| `%L`   | 日志级别（大写）   |
| `%l`   | 日志级别（小写）   |
| `%Y`   | 四位年份           |
| `%m`   | 月份               |
| `%d`   | 日期               |
| `%H`   | 小时（24小时制）   |
| `%M`   | 分钟               |
| `%S`   | 秒                 |
| `%e`   | 毫秒               |
| `%f`   | 微秒               |
| `%F`   | 纳秒               |
| `%t`   | 线程ID             |
| `%P`   | 进程ID             |
| `%s`   | 源文件名           |
| `%#`   | 源码行号           |
| `%!`   | 源码函数名         |
| `%^` / `%$` | 颜色范围起止 |
| `%%`   | 字面量 `%`         |

完整列表请参考 [`PatternCharacter`](crate::fmt::parse::PatternCharacter) 文档。

## 📄 许可证

MIT

---

感谢使用 `multi_logging`！如果你遇到问题或有改进建议，欢迎随时交流。


[`log`]: https://crates.io/crates/log
[`FileSinkMT`]: crate::sink::FileSinkMT
[`FileSinkST`]: crate::sink::FileSinkST
[`StdoutSinkMT`]: crate::sink::StdoutSinkMT
[`StderrSinkMT`]: crate::sink::StdoutSinkST
[`NullSink`]: crate::sink::NullSink
[`HookSink`]: crate::sink::HookSink
[`crossbeam`]: https://crates.io/crates/crossbeam
[`set_global_format`]: crate::set_global_format
[`register_logger`]: crate::register_logger
[`register_sink`]: crate::register_sink
