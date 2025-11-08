//! A UTF-8 encoded, growable string.
//!
//! This module contains the [`String`] type, Aurora's owned, heap-allocated
//! string type. Unlike string slices (`&str`), `String` owns its data and
//! can grow and shrink dynamically.
//!
//! # Examples
//!
//! ```
//! use aurora_stdlib::string::String;
//!
//! let mut s = String::new();
//! s.push_str("hello");
//! s.push(' ');
//! s.push_str("world");
//! assert_eq!(s.as_str(), "hello world");
//! ```

/// Re-export of `std::string::String` for Aurora programs.
///
/// See the [standard library documentation](https://doc.rust-lang.org/std/string/struct.String.html)
/// for full API reference.
pub use std::string::String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = String::new();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_from() {
        let s = String::from("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_push_str() {
        let mut s = String::new();
        s.push_str("hello");
        assert_eq!(s.as_str(), "hello");
        s.push_str(" ");
        s.push_str("world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_push() {
        let mut s = String::from("abc");
        s.push('1');
        s.push('2');
        s.push('3');
        assert_eq!(s.as_str(), "abc123");
    }

    #[test]
    fn test_contains() {
        let s = String::from("hello world");
        assert!(s.contains("world"));
        assert!(s.contains("hello"));
        assert!(!s.contains("foo"));
    }

    #[test]
    fn test_clone() {
        let s1 = String::from("hello");
        let s2 = s1.clone();
        assert_eq!(s1.as_str(), s2.as_str());
    }
}
