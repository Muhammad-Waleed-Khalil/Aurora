//! Basic I/O functionality for Aurora
//!
//! This module provides fundamental I/O operations like printing to stdout and stderr.

use std::io::{self, Write};

/// Prints to the standard output.
///
/// # Examples
///
/// ```
/// use aurora_stdlib::io::print;
/// print("Hello world!");
/// ```
pub fn print(s: &str) {
    let _ = io::stdout().write_all(s.as_bytes());
    let _ = io::stdout().flush();
}

/// Prints to the standard output, with a newline.
///
/// # Examples
///
/// ```
/// use aurora_stdlib::io::println;
/// println("Hello, world!");
/// ```
pub fn println(s: &str) {
    let _ = writeln!(io::stdout(), "{}", s);
}

/// Prints to the standard error.
///
/// # Examples
///
/// ```
/// use aurora_stdlib::io::eprint;
/// eprint("Error!");
/// ```
pub fn eprint(s: &str) {
    let _ = io::stderr().write_all(s.as_bytes());
    let _ = io::stderr().flush();
}

/// Prints to the standard error, with a newline.
///
/// # Examples
///
/// ```
/// use aurora_stdlib::io::eprintln;
/// eprintln("Error: something went wrong!");
/// ```
pub fn eprintln(s: &str) {
    let _ = writeln!(io::stderr(), "{}", s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        // Just ensure these don't panic
        print("test");
        println("test");
    }

    #[test]
    fn test_error_output() {
        eprint("error");
        eprintln("error");
    }
}
