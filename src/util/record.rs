//! 每条日志的源信息。
//!
//! 是来自 [`log`] crate 的类型的轻量封装。

use std::time::SystemTime;

// 几个可重复复用数据，减少获取开销。
thread_local! {
    /// 当前进程id。
    static CURRENT_PROCESS_ID: u32 = std::process::id();

    /// 当前线程id。
    ///
    /// SAFETY: ThreadId内部只有一个唯一的 NonZeroU64 字段。
    static CURRENT_THREAD_ID: u64 = unsafe { std::mem::transmute::<std::thread::ThreadId,std::num::NonZero<u64>>(std::thread::current().id()).get() };
}

macro_rules! impl_splice {
    ($tmp_string:expr, $tmp_offset:expr, $record:expr, $name:expr) => {
        $tmp_string.push_str($name.as_ref());
        $tmp_offset[0] = $name.as_ref().len();

        $tmp_string.push_str($record.target());
        $tmp_offset[1] = $tmp_offset[0] + $record.target().len();

        if let Some(path) = $record.module_path() {
            $tmp_string.push_str(path);
            $tmp_offset[2] = $tmp_offset[1] + path.len();
        }

        if let Some(file) = $record.file() {
            $tmp_string.push_str(file);
            $tmp_offset[3] = $tmp_offset[2] + file.len();
        }
    };
}

/// 日志仅经过拼接，但还没有格式化的所有数据。
#[derive(Clone)]
pub struct Record {
    // 包含多个数据拼接后的结果。
    data: String,
    // 各个数据的偏移量。
    // logger_name - target - module_path - file - log_detail - kv
    #[cfg(feature = "kv")]
    offsets: [usize; 5],
    #[cfg(not(feature = "kv"))]
    offsets: [usize; 4],
    line: Option<u32>,
    time_stamp: SystemTime,
    level: log::Level,
    process_id: u32,
    thread_id: u64,
}

impl Record {
    const TARGET_OFFSET: usize = 0;
    const MODULE_PATH_OFFSET: usize = 1;
    const FILE_OFFSET: usize = 2;
    const LOG_DETAIL_OFFSET: usize = 3;
    #[allow(unused)]
    const KV_OFFSET: usize = 4;

    #[inline(always)]
    pub fn new(record: &log::Record<'_>, name: impl AsRef<str>) -> Self {
        let (offsets, res_string) = Self::splice(record, name).expect("Kv Error");
        Self {
            data: res_string,
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
        #[cfg(feature = "kv")]
        return &self.data[self.offsets[Self::LOG_DETAIL_OFFSET]..self.offsets[Self::KV_OFFSET]];
        #[cfg(not(feature = "kv"))]
        &self.data[self.offsets[Self::LOG_DETAIL_OFFSET]..]
    }

    #[inline(always)]
    pub fn logger_name(&self) -> &str {
        &self.data[..self.offsets[Self::TARGET_OFFSET]]
    }

    #[inline(always)]
    pub fn file(&self) -> Option<&str> {
        if self.offsets[Self::FILE_OFFSET] == self.offsets[Self::LOG_DETAIL_OFFSET] {
            None
        } else {
            Some(&self.data[self.offsets[Self::FILE_OFFSET]..self.offsets[Self::LOG_DETAIL_OFFSET]])
        }
    }

    #[inline(always)]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    #[inline(always)]
    pub fn module_path(&self) -> Option<&str> {
        if self.offsets[Self::MODULE_PATH_OFFSET] == self.offsets[Self::FILE_OFFSET] {
            None
        } else {
            Some(
                &self.data[self.offsets[Self::MODULE_PATH_OFFSET]..self.offsets[Self::FILE_OFFSET]],
            )
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
        &self.data[self.offsets[Self::TARGET_OFFSET]..self.offsets[Self::MODULE_PATH_OFFSET]]
    }

    #[inline(always)]
    pub fn time_stamp_nano(&self) -> u128 {
        self.time_stamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    #[inline(always)]
    pub fn time_stamp(&self) -> SystemTime {
        self.time_stamp
    }

    #[inline(always)]
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    /// 线程ID来自于 [`ThreadId`](std::thread::ThreadId)，与 os线程id 往往不同。
    #[inline(always)]
    pub fn thread_id(&self) -> u64 {
        self.thread_id
    }

    #[cfg(feature = "kv")]
    #[inline(always)]
    pub fn key_values(&self) -> &str {
        &self.data[self.offsets[Self::KV_OFFSET]..]
    }

    /// 拼接 [`log::Record`] 中的零散字符切片为一整个字符串。
    ///
    /// 返回值为 (offsets, res_string)
    #[inline(always)]
    #[cfg(feature = "kv")]
    fn splice(
        record: &log::Record<'_>,
        name: impl AsRef<str>,
    ) -> Result<([usize; 5], String), log::kv::Error> {
        use core::fmt::Write;

        struct KvVister<'a> {
            tmp_string: &'a mut String,
            is_first: bool,
        }

        impl<'kvs> log::kv::VisitSource<'kvs> for KvVister<'kvs> {
            fn visit_pair(
                &mut self,
                key: log::kv::Key<'kvs>,
                value: log::kv::Value<'kvs>,
            ) -> Result<(), log::kv::Error> {
                if !self.is_first {
                    self.tmp_string.push(' ');
                }
                self.is_first = false;
                write!(self.tmp_string, "{}={}", key, value).map_err(log::kv::Error::from)
            }
        }

        let mut tmp_offset = [0; 5];
        let mut tmp_string = crate::global::acquire_string();

        impl_splice!(tmp_string, tmp_offset, record, name);

        let _ = write!(tmp_string, "{}", record.args());
        tmp_offset[4] = tmp_string.len();

        record.key_values().visit(&mut KvVister {
            tmp_string: &mut tmp_string,
            is_first: true,
        })?;

        Ok((tmp_offset, tmp_string))
    }

    /// 拼接 [`log::Record`] 中的零散字符切片为一整个字符串。
    ///
    /// 返回值为 (offsets, res_string)
    #[inline(always)]
    #[cfg(not(feature = "kv"))]
    fn splice(record: &log::Record<'_>, name: impl AsRef<str>) -> Result<([usize; 4], String), ()> {
        use core::fmt::Write;

        let mut tmp_offset = [0; 4];
        let mut tmp_string = crate::global::acquire_string();

        impl_splice!(tmp_string, tmp_offset, record, name);

        let _ = write!(tmp_string, "{}", record.args());

        Ok((tmp_offset, tmp_string))
    }
}
