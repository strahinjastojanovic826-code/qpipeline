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