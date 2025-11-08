//! Aurora Standard Library
//!
//! This library provides the fundamental building blocks for Aurora programs,
//! including core types like `Option`, `Result`, and `String`, as well as
//! I/O functionality.
//!
//! # Core Types
//!
//! - [`Option<T>`] - Type for values that may or may not exist
//! - [`Result<T, E>`] - Type for operations that may fail
//! - [`String`] - UTF-8 encoded, growable string
//!
//! # Examples
//!
//! ```
//! use aurora_stdlib::prelude::*;
//!
//! fn main() {
//!     let x = Some(42);
//!     println!("The answer is {}", x.unwrap());
//! }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod option;
pub mod result;
pub mod string;
pub mod io;

// Re-export commonly used types
pub use option::Option;
pub use result::Result;
pub use string::String;

/// The Aurora Prelude
///
/// Import everything you need to get started with Aurora:
///
/// ```
/// use aurora_stdlib::prelude::*;
/// ```
pub mod prelude {
    pub use crate::option::Option::{self, None, Some};
    pub use crate::result::Result::{self, Ok, Err};
    pub use crate::string::String;
    pub use crate::io::{print, println};
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use prelude::*;

    #[test]
    fn test_prelude_option() {
        let x: Option<i32> = Some(42);
        assert_eq!(x.unwrap(), 42);
    }

    #[test]
    fn test_prelude_result() {
        let y: Result<i32, &str> = Ok(100);
        assert_eq!(y.unwrap(), 100);
    }

    #[test]
    fn test_prelude_string() {
        let s = String::from("hello");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_option_result_interaction() {
        let x: Option<i32> = Some(42);
        let y: Result<i32, &str> = Ok(x.unwrap());
        assert_eq!(y.unwrap(), 42);

        let z: Option<i32> = y.ok();
        assert_eq!(z, Some(42));
    }
}
