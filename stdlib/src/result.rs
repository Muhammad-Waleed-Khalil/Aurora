//! Error handling with the `Result` type.
//!
//! `Result<T, E>` is the type used for returning and propagating errors. It is an enum
//! with the variants `Ok(T)`, representing success and containing a value, and `Err(E)`,
//! representing error and containing an error value.
//!
//! # Examples
//!
//! ```
//! use aurora_stdlib::result::Result::{self, Ok, Err};
//!
//! fn divide(a: i32, b: i32) -> Result<i32, String> {
//!     if b == 0 {
//!         Err(String::from("division by zero"))
//!     } else {
//!         Ok(a / b)
//!     }
//! }
//! ```

/// Re-export of `std::result::Result` for Aurora programs.
///
/// See the [standard library documentation](https://doc.rust-lang.org/std/result/enum.Result.html)
/// for full API reference.
pub use std::result::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use Result::*;

    #[test]
    fn test_is_ok_is_err() {
        let x: Result<i32, &str> = Ok(2);
        assert!(x.is_ok());
        assert!(!x.is_err());

        let x: Result<i32, &str> = Err("error");
        assert!(!x.is_ok());
        assert!(x.is_err());
    }

    #[test]
    fn test_ok() {
        let x: Result<u32, &str> = Ok(2);
        assert_eq!(x.ok(), Some(2));

        let x: Result<u32, &str> = Err("Nothing here");
        assert_eq!(x.ok(), None);
    }

    #[test]
    fn test_err() {
        let x: Result<u32, &str> = Ok(2);
        assert_eq!(x.err(), None);

        let x: Result<u32, &str> = Err("Nothing here");
        assert_eq!(x.err(), Some("Nothing here"));
    }

    #[test]
    fn test_unwrap() {
        let x: Result<u32, &str> = Ok(2);
        assert_eq!(x.unwrap(), 2);
    }

    #[test]
    #[should_panic]
    fn test_unwrap_panic() {
        let x: Result<u32, &str> = Err("error");
        x.unwrap();
    }

    #[test]
    fn test_unwrap_or() {
        let default = 2;
        let x: Result<u32, &str> = Ok(9);
        assert_eq!(x.unwrap_or(default), 9);

        let x: Result<u32, &str> = Err("error");
        assert_eq!(x.unwrap_or(default), default);
    }

    #[test]
    fn test_map() {
        let x: Result<u32, &str> = Ok(2);
        assert_eq!(x.map(|v| v * 2), Ok(4));

        let x: Result<u32, &str> = Err("error");
        assert_eq!(x.map(|v| v * 2), Err("error"));
    }

    #[test]
    fn test_and_then() {
        let sq = |x: u32| -> Result<u32, &'static str> { Ok(x * x) };
        assert_eq!(Ok(2).and_then(sq).and_then(sq), Ok(16));
        assert_eq!(Err("error").and_then(sq).and_then(sq), Err("error"));
    }
}
