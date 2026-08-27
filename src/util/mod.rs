//! 提供一些能在多处使用的api。
//!

pub mod dispatcher;
pub mod errors;
pub mod marker;
pub mod pool;
pub mod record;

pub use record::*;

/// 所有 sink 与 Logger 的默认名字。
pub const DEFAULT_NULL_NAME: &str = "__NULL_";
