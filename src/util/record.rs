//! 每条日志的源信息。
//!
//! 是来自 [`log`] crate 的类型的轻量封装。

use std::{thread::ThreadId, time::SystemTime};

// 几个可重复复用数据，减少获取开销。
thread_local! {
    /// 当前进程id。
    static CURRENT_PROCESS_ID: u32 = std::process::id();

    /// 当前线程id。
    static CURRENT_THREAD_ID: ThreadId = std::thread::current().id();
}

/// 日志仅经过拼接，但还没有格式化的所有数据。
#[derive(Clone)]
pub struct Record {
    data: String, // 包含多个数据拼接后的结果。
    logger_name: String,
    // 各个数据的源信息。
    // target(offset 省略) - module_path(offset) - file(offset) - spliced_string(offset)
    offsets: [usize; 3],
    line: Option<u32>,
    time_stamp: SystemTime,
    level: log::Level,
    process_id: u32,
    thread_id: ThreadId,
}

impl Record {
    #[inline(always)]
    pub fn new(record: &log::Record<'_>, name: impl AsRef<str>) -> Self {
        let (offsets, res_string) = Self::splice(record);
        let mut logger_name = crate::util::dispatcher::acquire_string();
        logger_name.push_str(name.as_ref());
        Self {
            data: res_string,
            logger_name,
            offsets,
            line: record.line(),
            time_stamp: SystemTime::now(),
            level: record.level(),
            process_id: CURRENT_PROCESS_ID.with(|id| *id),
            thread_id: CURRENT_THREAD_ID.with(|id| *id),
        }
    }

    #[inline(always)]
    pub fn log_detail(&self) -> &str {
        &self.data[self.offsets[2]..]
    }

    #[inline(always)]
    pub fn logger_name(&self) -> &str {
        &self.logger_name
    }

    #[inline(always)]
    pub fn file(&self) -> Option<&str> {
        if self.offsets[1] == self.offsets[2] {
            None
        } else {
            Some(&self.data[self.offsets[1]..self.offsets[2]])
        }
    }

    #[inline(always)]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    #[inline(always)]
    pub fn module_path(&self) -> Option<&str> {
        if self.offsets[0] == self.offsets[1] {
            None
        } else {
            Some(&self.data[self.offsets[0]..self.offsets[1]])
        }
    }

    #[inline(always)]
    pub fn metadata(&self) -> log::Metadata<'_> {
        log::MetadataBuilder::new()
            .level(self.level)
            .target(self.target())
            .build()
    }

    #[inline(always)]
    pub fn level(&self) -> log::Level {
        self.level
    }

    #[inline(always)]
    pub fn target(&self) -> &str {
        &self.data[..self.offsets[0]]
    }

    #[inline(always)]
    pub fn time_stamp_nano(&self) -> u128 {
        self.time_stamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    #[inline(always)]
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    #[inline(always)]
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    /// 拼接 [`log::Record`] 中的零散字符切片为一整个字符串。
    ///
    /// 返回值为 (offsets, res_string)
    #[inline(always)]
    fn splice(record: &log::Record<'_>) -> ([usize; 3], String) {
        use core::fmt::Write;

        let mut tmp_offset = [0; 3];
        let mut tmp_string = crate::util::dispatcher::acquire_string();

        tmp_string.push_str(record.target());
        tmp_offset[0] = record.target().len();

        if let Some(path) = record.module_path() {
            tmp_string.push_str(path);
            tmp_offset[1] = tmp_offset[0] + path.len();
        }

        if let Some(file) = record.file() {
            tmp_string.push_str(file);
            tmp_offset[2] = tmp_offset[1] + file.len();
        }

        let _ = write!(tmp_string, "{}", record.args());

        (tmp_offset, tmp_string)
    }
}
