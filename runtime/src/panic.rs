//! Panic handler for Aurora runtime
//!
//! Handles panic situations with backtrace generation and error reporting.

use std::backtrace::Backtrace;
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to prevent recursive panics
static PANICKING: AtomicBool = AtomicBool::new(false);

/// Panic information structure
#[repr(C)]
pub struct PanicInfo {
    pub message: *const u8,
    pub message_len: usize,
    pub file: *const u8,
    pub file_len: usize,
    pub line: u32,
    pub column: u32,
}

/// Main panic handler
///
/// # Safety
/// This function is unsafe because it accesses raw pointers from PanicInfo.
/// The caller must ensure all pointers are valid.
#[no_mangle]
pub unsafe extern "C" fn aurora_panic(info: *const PanicInfo) -> ! {
    // Check for recursive panic
    if PANICKING.swap(true, Ordering::SeqCst) {
        eprintln!("\n!!! DOUBLE PANIC !!!\nAborting immediately.");
        process::abort();
    }

    let info = &*info;

    // Extract panic message
    let message = if info.message.is_null() {
        "explicit panic"
    } else {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            info.message,
            info.message_len,
        ))
    };

    // Extract file location
    let file = if info.file.is_null() {
        "<unknown>"
    } else {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(info.file, info.file_len))
    };

    // Print panic message
    let _ = writeln!(
        io::stderr(),
        "\n{red}thread 'main' panicked at {file}:{line}:{col}:{reset}\n{message}",
        red = "\x1b[31;1m",
        reset = "\x1b[0m",
        file = file,
        line = info.line,
        col = info.column,
        message = message,
    );

    // Print backtrace
    print_backtrace();

    // Print note
    let _ = writeln!(
        io::stderr(),
        "{dim}note: run with `AURORA_BACKTRACE=1` for more detailed backtrace{reset}",
        dim = "\x1b[2m",
        reset = "\x1b[0m",
    );

    // Abort
    process::abort();
}

/// Print backtrace to stderr
fn print_backtrace() {
    use std::env;

    // Check if backtrace is enabled
    let backtrace_enabled = env::var("AURORA_BACKTRACE")
        .map(|v| v == "1" || v.to_lowercase() == "full")
        .unwrap_or(false);

    if !backtrace_enabled {
        return;
    }

    let backtrace = Backtrace::capture();

    match backtrace.status() {
        std::backtrace::BacktraceStatus::Captured => {
            let _ = writeln!(io::stderr(), "\nstack backtrace:");
            let _ = writeln!(io::stderr(), "{}", backtrace);
        }
        std::backtrace::BacktraceStatus::Disabled => {
            let _ = writeln!(
                io::stderr(),
                "\nnote: backtrace disabled, set AURORA_BACKTRACE=1 to enable"
            );
        }
        _ => {
            let _ = writeln!(io::stderr(), "\nnote: backtrace not available");
        }
    }
}

/// Panic with a simple message
#[no_mangle]
pub extern "C" fn aurora_panic_msg(msg: *const u8, msg_len: usize) -> ! {
    let info = PanicInfo {
        message: msg,
        message_len: msg_len,
        file: std::ptr::null(),
        file_len: 0,
        line: 0,
        column: 0,
    };

    unsafe {
        aurora_panic(&info);
    }
}

/// Panic with bounds check failure
#[no_mangle]
pub extern "C" fn aurora_panic_bounds_check(
    index: usize,
    len: usize,
    file: *const u8,
    file_len: usize,
    line: u32,
) -> ! {
    let mut msg = String::new();
    let _ = write!(
        msg,
        "index out of bounds: the len is {} but the index is {}",
        len, index
    );

    let msg_bytes = msg.as_bytes();
    let info = PanicInfo {
        message: msg_bytes.as_ptr(),
        message_len: msg_bytes.len(),
        file,
        file_len,
        line,
        column: 0,
    };

    unsafe {
        aurora_panic(&info);
    }
}

/// Panic with None unwrap
#[no_mangle]
pub extern "C" fn aurora_panic_unwrap_none(
    file: *const u8,
    file_len: usize,
    line: u32,
) -> ! {
    let msg = b"called `Option::unwrap()` on a `None` value";

    let info = PanicInfo {
        message: msg.as_ptr(),
        message_len: msg.len(),
        file,
        file_len,
        line,
        column: 0,
    };

    unsafe {
        aurora_panic(&info);
    }
}

/// Panic with Err unwrap
#[no_mangle]
pub extern "C" fn aurora_panic_unwrap_err(
    err_msg: *const u8,
    err_msg_len: usize,
    file: *const u8,
    file_len: usize,
    line: u32,
) -> ! {
    let err = unsafe {
        if err_msg.is_null() {
            "<error>"
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(err_msg, err_msg_len))
        }
    };

    let mut msg = String::new();
    let _ = write!(msg, "called `Result::unwrap()` on an `Err` value: {}", err);

    let msg_bytes = msg.as_bytes();
    let info = PanicInfo {
        message: msg_bytes.as_ptr(),
        message_len: msg_bytes.len(),
        file,
        file_len,
        line,
        column: 0,
    };

    unsafe {
        aurora_panic(&info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "test panic")]
    fn test_panic_msg() {
        let msg = b"test panic";
        aurora_panic_msg(msg.as_ptr(), msg.len());
    }

    // Note: Can't test actual panic without abort, but we can test the functions compile
    #[test]
    fn test_panic_compiles() {
        // Just verify the functions exist and are callable (won't actually panic)
        let _ = PANICKING.load(Ordering::SeqCst);
    }
}
