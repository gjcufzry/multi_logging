//! 提供一些能在多处使用的api。
//!

pub(crate) mod dispatcher;
pub(crate) mod errors;
pub(crate) mod marker;
pub(crate) mod pool;
pub(crate) mod record;

pub use record::*;

/// 所有 sink 与 Logger 的默认名字。
pub const DEFAULT_NULL_NAME: &str = "__NULL_";
