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