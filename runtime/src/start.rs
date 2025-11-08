//! Program initialization and startup
//!
//! Handles program entry point and initialization before main().

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Program arguments structure
#[repr(C)]
pub struct ProgramArgs {
    pub argc: c_int,
    pub argv: *const *const c_char,
    pub envp: *const *const c_char,
}

/// Main function signature for Aurora programs
pub type AuroraMainFn = unsafe extern "C" fn() -> c_int;

/// Program entry point wrapper
///
/// This is called by the system _start function or CRT.
/// It initializes the runtime and calls the Aurora main function.
///
/// # Safety
/// This function is unsafe because it's the entry point and handles raw pointers.
#[no_mangle]
pub unsafe extern "C" fn aurora_start(
    argc: c_int,
    argv: *const *const c_char,
    envp: *const *const c_char,
    main_fn: AuroraMainFn,
) -> c_int {
    // Initialize runtime
    initialize_runtime(argc, argv, envp);

    // Call user's main function
    let exit_code = main_fn();

    // Cleanup runtime
    cleanup_runtime();

    exit_code
}

/// Initialize the Aurora runtime
unsafe fn initialize_runtime(argc: c_int, argv: *const *const c_char, envp: *const *const c_char) {
    // Set up program arguments
    set_program_args(argc, argv, envp);

    // Initialize thread-local storage
    #[cfg(feature = "tls")]
    crate::tls::initialize_tls();

    // Set up signal handlers (if needed)
    #[cfg(unix)]
    setup_signal_handlers();
}

/// Cleanup the Aurora runtime
fn cleanup_runtime() {
    // Run exit handlers
    run_exit_handlers();

    // Print allocation stats in debug mode
    #[cfg(feature = "debug_allocator")]
    {
        let (count, bytes) = crate::allocator::aurora_alloc_stats();
        if count > 0 {
            eprintln!(
                "\n[Aurora Debug] Memory leak detected: {} allocations, {} bytes",
                count, bytes
            );
        }
    }
}

/// Global storage for program arguments
static mut PROGRAM_ARGS: Option<ProgramArgs> = None;

/// Set program arguments
unsafe fn set_program_args(argc: c_int, argv: *const *const c_char, envp: *const *const c_char) {
    PROGRAM_ARGS = Some(ProgramArgs { argc, argv, envp });
}

/// Get program arguments
///
/// Returns None if called before aurora_start
#[no_mangle]
pub extern "C" fn aurora_get_args() -> Option<&'static ProgramArgs> {
    unsafe { PROGRAM_ARGS.as_ref() }
}

/// Get program argument by index
///
/// # Safety
/// Index must be < argc
#[no_mangle]
pub unsafe extern "C" fn aurora_get_arg(index: c_int) -> *const c_char {
    if let Some(args) = PROGRAM_ARGS.as_ref() {
        if index >= 0 && index < args.argc {
            return *args.argv.offset(index as isize);
        }
    }
    std::ptr::null()
}

/// Get environment variable by name
///
/// # Safety
/// name must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn aurora_getenv(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return std::ptr::null();
    }

    let name_cstr = CStr::from_ptr(name);
    std::env::var_os(name_cstr.to_str().unwrap_or(""))
        .and_then(|s| s.into_string().ok())
        .map(|s| s.as_ptr() as *const c_char)
        .unwrap_or(std::ptr::null())
}

/// Exit handler function type
type ExitHandler = extern "C" fn();

/// Global storage for exit handlers
static mut EXIT_HANDLERS: Vec<ExitHandler> = Vec::new();

/// Register an exit handler
///
/// Handlers are called in reverse order (LIFO) during cleanup.
#[no_mangle]
pub extern "C" fn aurora_atexit(handler: ExitHandler) {
    unsafe {
        EXIT_HANDLERS.push(handler);
    }
}

/// Run all registered exit handlers
fn run_exit_handlers() {
    unsafe {
        while let Some(handler) = EXIT_HANDLERS.pop() {
            handler();
        }
    }
}

/// Set up signal handlers (Unix only)
#[cfg(unix)]
unsafe fn setup_signal_handlers() {
    use libc::{signal, SIGABRT, SIGFPE, SIGILL, SIGSEGV, SIG_DFL};

    // Reset signal handlers to default
    signal(SIGABRT, SIG_DFL);
    signal(SIGFPE, SIG_DFL);
    signal(SIGILL, SIG_DFL);
    signal(SIGSEGV, SIG_DFL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_handler_registration() {
        extern "C" fn test_handler() {}

        aurora_atexit(test_handler);

        unsafe {
            assert_eq!(EXIT_HANDLERS.len(), 1);
            EXIT_HANDLERS.clear(); // Clean up for other tests
        }
    }

    #[test]
    fn test_get_args_before_init() {
        assert!(aurora_get_args().is_none());
    }
}
