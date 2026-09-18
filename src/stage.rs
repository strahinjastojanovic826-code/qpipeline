use crate::chunk::QuatChunk;

/// Osnovna osobina za svaku fazu u pipeline-u
pub trait QuatStage: Send + Sync {
    fn process(&self, chunk: QuatChunk) -> QuatChunk;
}

/// Faza: Invertovanje svih stanja u bloku
pub struct QuatInvertStage;

impl QuatStage for QuatInvertStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> QuatChunk {
        QuatChunk(!chunk.0)
    }
}

/// Faza: Shift udesno za N kvat mesta
pub struct QuatShiftRightStage {
    pub quat_places: usize,
}

impl QuatStage for QuatShiftRightStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> QuatChunk {
        QuatChunk(chunk.0 >> (self.quat_places * 2))
    }
}

/// Faza: XOR maska nad blokom
pub struct QuatXorMaskStage {
    pub mask: u64,
}

impl QuatStage for QuatXorMaskStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> QuatChunk {
        QuatChunk(chunk.0 ^ self.mask)
    }
}