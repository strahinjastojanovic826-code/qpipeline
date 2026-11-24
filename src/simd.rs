use crate::chunk::QuatChunk;

pub trait SimdQuatBatch {
    fn simd_invert(chunks: &mut [QuatChunk]);
    fn simd_xor_mask(chunks: &mut [QuatChunk], mask: u64);
}

pub struct SimdEngine;

impl SimdEngine {
    /// Inverts quaternary states across multiple chunks using AVX2 instructions (x86_64)
    #[cfg(target_arch = "x86_64")]
    pub fn invert_avx2(chunks: &mut [QuatChunk]) {
        #[cfg(target_feature = "avx2")]
        use std::arch::x86_64::*;

        let mut i = 0;
        let len = chunks.len();

        // Process 4 chunks (256 bits = 128 quaternary states) simultaneously
        #[cfg(target_feature = "avx2")]
        // SAFETY:
        // - `i + 4 <= len` guarantees at least 4 initialized `QuatChunk` elements (4 * 64 bits = 256 bits)
        //   starting at `chunks[i..].as_mut_ptr()`, with valid mutable provenance and non-overlapping access.
        // - Casting `*mut QuatChunk` to `*mut __m256i` is valid under the assumption `QuatChunk` has a single
        //   `u64` layout (`#[repr(transparent)]`) and we use unaligned load/store (`_mm256_loadu_si256` /
        //   `_mm256_storeu_si256`), so 32-byte alignment of `ptr` is not required.
        // - Bitwise XOR with `_mm256_set1_epi64x(-1)` (all `1` bits) performs a 256-bit bitwise NOT,
        //   inverting all 128 packed 2-bit quaternary states (`00 <-> 11`, `01 <-> 10`) in one vector op.
        // - Caller/dispatch must ensure AVX2 is supported on the target CPU before executing these intrinsics.
        unsafe {
            while i + 4 <= len {
                let ptr = chunks[i..].as_mut_ptr() as *mut __m256i;
                let data = _mm256_loadu_si256(ptr);
                // Bitwise NOT operation across entire 256-bit register
                let inverted = _mm256_xor_si256(data, _mm256_set1_epi64x(-1));
                _mm256_storeu_si256(ptr, inverted);
                i += 4;
            }
        }

        // Fallback for remaining scalar elements
        while i < len {
            chunks[i] = chunks[i].invert();
            i += 1;
        }
    }

    /// Software fallback vectorization (auto-vectorizable by LLVM on ARM/x86)
    pub fn invert_portable(chunks: &mut [QuatChunk]) {
        for chunk in chunks.iter_mut() {
            chunk.0 = !chunk.0;
        }
    }
}