//! 标记类型与traits。

use std::cell::UnsafeCell;

use crossbeam::atomic::AtomicCell;

/// 一个可能存在的锁。
///
/// 用于静态实现不同的同步策略。
pub trait MaybeMutexType: Send + Sync {
    /// 能够在被析构时自动释放的守卫。
    type Guard<'a>
    where
        Self: 'a;

    /// 需要的内部值（使用时只会使用 `()`）。
    type Inner;

    fn may_mutex_lock<'a>(&'a self) -> Self::Guard<'a>;

    fn may_mutex_new(val: Self::Inner) -> Self;
}

impl<T: Send> MaybeMutexType for std::sync::Mutex<T> {
    type Guard<'a>
        = std::sync::MutexGuard<'a, T>
    where
        Self: 'a;

    type Inner = T;

    /// 获取一个锁。
    #[inline]
    fn may_mutex_lock<'a>(&'a self) -> Self::Guard<'a> {
        self.lock().unwrap()
    }

    #[inline]
    fn may_mutex_new(val: Self::Inner) -> Self {
        Self::new(val)
    }
}

impl MaybeMutexType for () {
    type Guard<'a>
        = ()
    where
        Self: 'a;

    type Inner = ();

    /// 函数什么也不会做。
    #[inline]
    fn may_mutex_lock<'a>(&'a self) -> Self::Guard<'a> {}

    #[inline]
    fn may_mutex_new(_val: Self::Inner) -> Self {}
}

/// 一个可能是原子地读取的 trait。
///
/// 用于静态实现不同的同步策略。
pub trait MaybeAtomicOperation<Inner>: Send + Sync {
    fn may_atomic_new(val: Inner) -> Self;

    fn may_atomic_load(&self) -> Inner;

    fn may_atomic_store(&self, val: Inner);
}

/// 原子地读取内部元素。
pub struct AtomicOperation<T>(AtomicCell<T>);

/// 非原子地读取内部元素。
///
/// SAFETY:
///
/// - 类型假设只会在单线程进行读写，并发读写是毫无疑问的未定义行为。
pub struct NoAtomicOperation<T>(UnsafeCell<T>);

impl<T: Copy + std::marker::Send> MaybeAtomicOperation<T> for AtomicOperation<T> {
    fn may_atomic_new(val: T) -> Self {
        Self(AtomicCell::new(val))
    }

    fn may_atomic_load(&self) -> T {
        self.0.load()
    }

    fn may_atomic_store(&self, val: T) {
        self.0.store(val);
    }
}

impl<T: Copy + std::marker::Send> MaybeAtomicOperation<T> for NoAtomicOperation<T> {
    fn may_atomic_new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    fn may_atomic_load(&self) -> T {
        // SAFETY: 类型期望单线程环境，在这种情况下是安全的。
        unsafe { *self.0.get() }
    }

    fn may_atomic_store(&self, val: T) {
        // SAFETY: 类型期望单线程环境，在这种情况下是安全的。
        unsafe { (*self.0.get()) = val }
    }
}

/// SAFETY: 类型假设只会在单线程进行读写，并发读写是毫无疑问的未定义行为。
unsafe impl<T> Sync for NoAtomicOperation<T> {}

/// 一个标记空类型，实现了 [`MaybeAtomicType`]，其关联类型用于原子操作。
pub struct AtomicType;

/// 一个标记空类型，实现了 [`MaybeAtomicType`]，其关联类型用于非原子操作。
///
/// SAFETY:
///
/// - 类型假设只会在单线程进行读写，并发读写是毫无疑问的未定义行为。
pub struct NoAtomicType;

/// 标记 trait，用于绑定类型且可以使用泛型参数。
/// 关联类型 `Inner` 可用于泛型。
pub trait MaybeAtomicType<T> {
    type Inner<U>: MaybeAtomicOperation<T>;
}

impl<T: Copy + std::marker::Send> MaybeAtomicType<T> for AtomicType {
    type Inner<U> = AtomicOperation<T>;
}

impl<T: Copy + std::marker::Send> MaybeAtomicType<T> for NoAtomicType {
    type Inner<U> = NoAtomicOperation<T>;
}
