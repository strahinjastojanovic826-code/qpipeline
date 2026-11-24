use crate::chunk::{QuatChunk, QuatState};
use crate::error::PipelineError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub trait QuatStage: Send + Sync {
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError>;

    /// Batch execution hook; stages can override this for faster slice processing
    fn process_batch(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        for chunk in chunks.iter_mut() {
            *chunk = self.process(*chunk)?;
        }
        Ok(())
    }
}

/// Statički cjevovod sa 0-cost compile-time spajanjem.
pub struct StaticPipeline<A, B> {
    pub stage_a: A,
    pub stage_b: B,
}

impl<A, B> QuatStage for StaticPipeline<A, B>
where
    A: QuatStage,
    B: QuatStage,
{
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        let intermediate = self.stage_a.process(chunk)?;
        self.stage_b.process(intermediate)
    }
}

pub trait QuatStageExt: QuatStage + Sized {
    #[inline(always)]
    fn then<Next: QuatStage>(self, next: Next) -> StaticPipeline<Self, Next> {
        StaticPipeline {
            stage_a: self,
            stage_b: next,
        }
    }
}

impl<T: QuatStage> QuatStageExt for T {}

// --- UGRAĐENE FAZE ---

#[derive(Debug, Clone, Copy, Default)]
pub struct QuatInvertStage;

impl QuatStage for QuatInvertStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        Ok(chunk.invert())
    }

    #[inline]
    fn process_batch(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                crate::simd::SimdEngine::invert_avx2(chunks);
                return Ok(());
            }
        }
        crate::simd::SimdEngine::invert_portable(chunks);
        Ok(())
    }

}

#[derive(Debug, Clone, Copy, Default)]
pub struct QuatShiftRightStage {
    pub quat_places: usize,
}

impl QuatStage for QuatShiftRightStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        Ok(QuatChunk(chunk.0 >> (self.quat_places * 2)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct QuatXorMaskStage {
    pub mask: u64,
}

impl QuatStage for QuatXorMaskStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        Ok(QuatChunk(chunk.0 ^ self.mask))
    }

    #[inline]
    fn process_batch(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        for chunk in chunks.iter_mut() {
            chunk.0 ^= self.mask;
        }
        Ok(())
    }

}

#[derive(Debug, Clone)]
pub struct QuatInspectStage {
    pub target: QuatState,
    pub counter: Arc<AtomicUsize>,
}

impl QuatInspectStage {
    pub fn new(target: QuatState) -> Self {
        Self {
            target,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn count(&self) -> usize {
        self.counter.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.counter.store(0, Ordering::Release);
    }
}

impl QuatStage for QuatInspectStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        let count = chunk.count_state(self.target);
        self.counter.fetch_add(count, Ordering::AcqRel);
        Ok(chunk)
    }
}

#[derive(Debug, Clone)]
pub struct QuatLookupStage {
    pub table: [QuatState; 4],
    byte_lut: [u8; 256],
}

impl QuatLookupStage {
    pub const fn new(table: [QuatState; 4]) -> Self {
        let mut byte_lut = [0u8; 256];
        let mut b: usize = 0;
        while b < 256 {
            let s0 = table[b & 0b11] as u8;
            let s1 = table[(b >> 2) & 0b11] as u8;
            let s2 = table[(b >> 4) & 0b11] as u8;
            let s3 = table[(b >> 6) & 0b11] as u8;
            byte_lut[b] = s0 | (s1 << 2) | (s2 << 4) | (s3 << 6);
            b += 1;
        }
        Self { table, byte_lut }
    }

    #[inline(always)]
    fn map_raw(&self, raw: u64) -> u64 {
        let bytes = raw.to_ne_bytes();
        let mapped_bytes = [
            self.byte_lut[bytes[0] as usize],
            self.byte_lut[bytes[1] as usize],
            self.byte_lut[bytes[2] as usize],
            self.byte_lut[bytes[3] as usize],
            self.byte_lut[bytes[4] as usize],
            self.byte_lut[bytes[5] as usize],
            self.byte_lut[bytes[6] as usize],
            self.byte_lut[bytes[7] as usize],
        ];
        u64::from_ne_bytes(mapped_bytes)
    }
}

impl QuatStage for QuatLookupStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        Ok(QuatChunk(self.map_raw(chunk.raw())))
    }

    #[inline]
    fn process_batch(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        for chunk in chunks.iter_mut() {
            chunk.0 = self.map_raw(chunk.0);
        }
        Ok(())
    }
}