//! # Foreign Function Interface (FFI) Safety & Invariants
//!
//! This module exposes a C-compatible (`extern "C"`) interface for managing and executing
//! a [`QuatPipeline`] from external callers (such as C, C++, or C# via P/Invoke).
//!
//! ## `unsafe` & ABI Invariants
//! - **Pointer Validity & Alignment**: Any non-null `ptr: *mut QuatPipeline` or `ptr: *const QuatPipeline`
//!   passed to these functions must point to a valid, properly aligned, and initialized
//!   `QuatPipeline` instance originally allocated via [`qpipeline_create`] (`Box::into_raw`).
//! - **Ownership & Aliasing**:
//!   - [`qpipeline_free`] transfers ownership back via `Box::from_raw` and drops the pipeline.
//!     It must be called at most once per handle, and the handle must never be used afterward.
//!   - [`qpipeline_add_invert_stage`] and [`qpipeline_add_xor_stage`] mutably borrow the underlying
//!     `QuatPipeline` (`&mut *ptr`) and must not be called concurrently with other operations on the
//!     same pipeline instance.
//! - **Raw Slice Construction (`qpipeline_execute_smart`)**:
//!   - When `len > 0`, `chunks_ptr` must be non-null, properly aligned for `QuatChunk` (`u64`),
//!     and point to at least `len` initialized elements that remain valid and unaliased for the
//!     duration of the call.
//!   - Casting `*mut u64` to `*mut QuatChunk` relies on `QuatChunk` having a compatible single-field
//!     layout; ensure `QuatChunk` uses `#[repr(transparent)]` for strict ABI guarantees.
//! - **Panic Boundaries**: Functions wrap operations in `catch_unwind` to prevent unwinding across the
//!   FFI boundary and map panics to [`QuatFFIStatus::PanicOccurred`].

use crate::builder::QuatPipeline;
use crate::chunk::QuatChunk;
use crate::stage::{QuatInvertStage, QuatXorMaskStage};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QuatFFIStatus {
    Ok = 0,
    NullPointer = 1,
    ExecutionError = 2,
    PanicOccurred = 3,
}

#[unsafe(no_mangle)]
pub extern "C" fn qpipeline_create() -> *mut QuatPipeline {
    let result = catch_unwind(|| Box::into_raw(Box::new(QuatPipeline::new())));
    result.unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qpipeline_free(ptr: *mut QuatPipeline) { unsafe {
    if ptr.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        drop(Box::from_raw(ptr));
    }));
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qpipeline_add_invert_stage(ptr: *mut QuatPipeline) -> QuatFFIStatus { unsafe {
    if ptr.is_null() {
        return QuatFFIStatus::NullPointer;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let pipeline = &mut *ptr;
        pipeline.push_stage(QuatInvertStage);
    }));

    match result {
        Ok(_) => QuatFFIStatus::Ok,
        Err(_) => QuatFFIStatus::PanicOccurred,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qpipeline_add_xor_stage(
    ptr: *mut QuatPipeline,
    mask: u64,
) -> QuatFFIStatus { unsafe {
    if ptr.is_null() {
        return QuatFFIStatus::NullPointer;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let pipeline = &mut *ptr;
        pipeline.push_stage(QuatXorMaskStage { mask });
    }));

    match result {
        Ok(_) => QuatFFIStatus::Ok,
        Err(_) => QuatFFIStatus::PanicOccurred,
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qpipeline_execute_smart(
    ptr: *const QuatPipeline,
    chunks_ptr: *mut u64,
    len: usize,
) -> QuatFFIStatus { unsafe {
    if ptr.is_null() || chunks_ptr.is_null() {
        return QuatFFIStatus::NullPointer;
    }

    if len == 0 {
        return QuatFFIStatus::Ok;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let pipeline = &*ptr;
        let raw_slice = slice::from_raw_parts_mut(chunks_ptr as *mut QuatChunk, len);
        pipeline.execute_smart(raw_slice)
    }));

    match result {
        Ok(Ok(_)) => QuatFFIStatus::Ok,
        Ok(Err(_)) => QuatFFIStatus::ExecutionError,
        Err(_) => QuatFFIStatus::PanicOccurred,
    }
}}