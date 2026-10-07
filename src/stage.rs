use crate::chunk::{QuatChunk, QuatState};
use crate::error::PipelineError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub trait QuatStage: Send + Sync {
    fn process(&self, chunk: QuatChunk) -> Result<QuatChunk, PipelineError>;
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