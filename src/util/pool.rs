//! 各种池。

use std::{sync::RwLock, thread::JoinHandle};

use crossbeam::{channel::Sender, queue::ArrayQueue};

use crate::logger::async_logger::LogCommand;

pub const DEFAULT_POOL_SIZE: usize = 1024 * 16;

/// 全局线程池。
///
/// 实际上就是一个用于等待线程结束的结构。
pub struct ThreadPool {
    inner: RwLock<Vec<(JoinHandle<()>, Sender<LogCommand>)>>,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Vec::new()),
        }
    }

    /// 添加一个线程句柄及其对应的发送者。
    #[inline]
    pub fn spawn(&self, handle: JoinHandle<()>, tx: Sender<LogCommand>) {
        self.inner.write().unwrap().push((handle, tx));
    }

    #[inline]
    pub fn exit_and_wait(&self) {
        let mut inner = self.inner.write().unwrap();
        // 先发送结束信息。
        for (_, s) in inner.iter() {
            let _ = s.send(LogCommand::Exit);
        }
        // 随后等待线程加入。
        while let Some((j, _)) = inner.pop() {
            let _ = j.join();
        }
    }
}

/// 全局对象池。
///
/// 主要用于复用 String。
pub struct ObjectPool {
    strings: ArrayQueue<String>, // 字符串缓冲区。
}

impl ObjectPool {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            strings: ArrayQueue::new(DEFAULT_POOL_SIZE),
        }
    }

    #[inline(always)]
    pub fn acquire_string(&self) -> String {
        self.strings
            .pop()
            .unwrap_or_else(|| String::with_capacity(512))
    }

    #[inline(always)]
    pub fn release_string(&self, obj: String) {
        // 队列满了就析构。
        let _ = self.strings.push(obj);
    }
}
