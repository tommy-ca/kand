pub mod error;
pub mod helper;
pub mod ta;

pub use error::KandError;
pub use paste;
pub use ta::ohlcv;
pub use ta::other;
pub use ta::stats;

/// Precision type for technical analysis calculations.
///
/// The type is determined by the enabled features:
/// - With feature "f32": Uses f32 (standard precision)
/// - With feature "f64": Uses f64 (high precision)
/// - With no features enabled: Defaults to f64
#[cfg(all(feature = "f32", not(feature = "f64")))]
pub type TAFloat = f32;

/// Precision type for technical analysis calculations.
///
/// The type is determined by the enabled features:
/// - With feature "f32": Uses f32 (standard precision)
/// - With feature "f64": Uses f64 (high precision)
/// - With no features enabled: Defaults to f64
#[cfg(not(all(feature = "f32", not(feature = "f64"))))]
pub type TAFloat = f64;

/// Integer type for technical analysis calculations.
///
/// The type is determined by the enabled features:
/// - With feature "i32": Uses i32 (standard precision)
/// - With feature "i64": Uses i64 (high precision)
/// - With no features enabled: Defaults to i64
#[cfg(all(feature = "i32", not(feature = "i64")))]
pub type TAInt = i32;

/// Integer type for technical analysis calculations.
///
/// The type is determined by the enabled features:
/// - With feature "i32": Uses i32 (standard precision)
/// - With feature "i64": Uses i64 (high precision)
/// - With no features enabled: Defaults to i64
#[cfg(not(all(feature = "i32", not(feature = "i64"))))]
pub type TAInt = i64;

/// Period type for technical analysis calculation parameters.
pub type TAPeriod = usize;

/// Standard epsilon values for floating-point comparisons.
///
/// The value is determined by the enabled features:
/// - f32: Uses f32::EPSILON (≈ 1.19e-7)
/// - f64: Uses f64::EPSILON (≈ 2.22e-16)
#[cfg(all(feature = "f32", not(feature = "f64")))]
pub const EPSILON: TAFloat = f32::EPSILON;

#[cfg(not(all(feature = "f32", not(feature = "f64"))))]
pub const EPSILON: TAFloat = f64::EPSILON;
