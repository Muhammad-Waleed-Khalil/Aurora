//! Memory allocator for Aurora runtime
//!
//! Provides malloc/free wrappers with optional debugging support.

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global allocation counter for debugging
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Allocate memory
///
/// # Safety
/// This function is unsafe because it returns uninitialized memory.
/// The caller must ensure proper initialization before use.
#[no_mangle]
pub unsafe extern "C" fn aurora_alloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }

    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    let ptr = alloc(layout);

    #[cfg(feature = "debug_allocator")]
    {
        if !ptr.is_null() {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(size, Ordering::Relaxed);
        }
    }

    ptr
}

/// Free memory
///
/// # Safety
/// The pointer must have been allocated by aurora_alloc with the same size and alignment.
#[no_mangle]
pub unsafe extern "C" fn aurora_free(ptr: *mut u8, size: usize, align: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return,
    };

    dealloc(ptr, layout);

    #[cfg(feature = "debug_allocator")]
    {
        ALLOC_COUNT.fetch_sub(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_sub(size, Ordering::Relaxed);
    }
}

/// Reallocate memory
///
/// # Safety
/// The pointer must have been allocated by aurora_alloc with old_size and align.
#[no_mangle]
pub unsafe extern "C" fn aurora_realloc(
    ptr: *mut u8,
    old_size: usize,
    new_size: usize,
    align: usize,
) -> *mut u8 {
    if ptr.is_null() {
        return aurora_alloc(new_size, align);
    }

    if new_size == 0 {
        aurora_free(ptr, old_size, align);
        return ptr::null_mut();
    }

    let old_layout = match Layout::from_size_align(old_size, align) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };

    let new_ptr = realloc(ptr, old_layout, new_size);

    #[cfg(feature = "debug_allocator")]
    {
        if !new_ptr.is_null() {
            ALLOC_BYTES.fetch_sub(old_size, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
    }

    new_ptr
}

/// Get allocation statistics (debug mode only)
#[no_mangle]
pub extern "C" fn aurora_alloc_stats() -> (usize, usize) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

/// Zero-fill allocated memory
///
/// # Safety
/// ptr must be valid for size bytes
#[no_mangle]
pub unsafe extern "C" fn aurora_alloc_zeroed(size: usize, align: usize) -> *mut u8 {
    let ptr = aurora_alloc(size, align);
    if !ptr.is_null() {
        ptr::write_bytes(ptr, 0, size);
    }
    ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_free() {
        unsafe {
            let ptr = aurora_alloc(1024, 8);
            assert!(!ptr.is_null());
            aurora_free(ptr, 1024, 8);
        }
    }

    #[test]
    fn test_alloc_zero_size() {
        unsafe {
            let ptr = aurora_alloc(0, 8);
            assert!(ptr.is_null());
        }
    }

    #[test]
    fn test_realloc_grow() {
        unsafe {
            let ptr = aurora_alloc(100, 8);
            assert!(!ptr.is_null());

            let new_ptr = aurora_realloc(ptr, 100, 200, 8);
            assert!(!new_ptr.is_null());

            aurora_free(new_ptr, 200, 8);
        }
    }

    #[test]
    fn test_realloc_shrink() {
        unsafe {
            let ptr = aurora_alloc(200, 8);
            assert!(!ptr.is_null());

            let new_ptr = aurora_realloc(ptr, 200, 100, 8);
            assert!(!new_ptr.is_null());

            aurora_free(new_ptr, 100, 8);
        }
    }

    #[test]
    fn test_alloc_zeroed() {
        unsafe {
            let ptr = aurora_alloc_zeroed(100, 8);
            assert!(!ptr.is_null());

            // Verify all bytes are zero
            for i in 0..100 {
                assert_eq!(*ptr.add(i), 0);
            }

            aurora_free(ptr, 100, 8);
        }
    }
}
