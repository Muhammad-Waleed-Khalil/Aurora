//! Option type for null-safety
//!
//! The `Option<T>` type represents an optional value: every `Option` is either
//! `Some` and contains a value, or `None`, and does not. This is Aurora's
//! primary way of handling the absence of values safely.
//!
//! # Examples
//!
//! ```
//! use aurora_stdlib::option::Option::{self, Some, None};
//!
//! let some_value = Some(42);
//! let no_value: Option<i32> = None;
//! ```

/// Re-export of `std::option::Option` for Aurora programs.
///
/// See the [standard library documentation](https://doc.rust-lang.org/std/option/enum.Option.html)
/// for full API reference.
pub use std::option::Option;

#[cfg(test)]
mod tests {
    use super::*;
    use Option::*;

    #[test]
    fn test_is_some_is_none() {
        let x: Option<i32> = Some(2);
        assert!(x.is_some());
        assert!(!x.is_none());

        let x: Option<i32> = None;
        assert!(!x.is_some());
        assert!(x.is_none());
    }

    #[test]
    fn test_unwrap() {
        let x = Some(42);
        assert_eq!(x.unwrap(), 42);
    }

    #[test]
    #[should_panic]
    fn test_unwrap_panic() {
        let x: Option<i32> = None;
        x.unwrap();
    }

    #[test]
    fn test_unwrap_or() {
        assert_eq!(Some(42).unwrap_or(0), 42);
        assert_eq!(None.unwrap_or(0), 0);
    }

    #[test]
    fn test_map() {
        let x = Some(2);
        assert_eq!(x.map(|v| v * 2), Some(4));

        let x: Option<i32> = None;
        assert_eq!(x.map(|v| v * 2), None);
    }

    #[test]
    fn test_and_then() {
        let sq = |x: i32| -> Option<i32> { Some(x * x) };
        assert_eq!(Some(2).and_then(sq).and_then(sq), Some(16));
        assert_eq!(None.and_then(sq).and_then(sq), None);
    }
}
