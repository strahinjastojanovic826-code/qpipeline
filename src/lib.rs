pub mod builder;
pub mod chunk;
pub mod stage;

pub use builder::QuatPipeline;
pub use chunk::{QuatChunk, QuatState};
pub use stage::{QuatInvertStage, QuatShiftRightStage, QuatStage, QuatXorMaskStage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_execution() {
        let pipeline = QuatPipeline::new()
            .add_stage(QuatInvertStage)
            .add_stage(QuatXorMaskStage { mask: 0x00FF_00FF });

        let chunk = QuatChunk::new(0x0000_0000);
        let result = pipeline.execute_chunk(chunk);

        assert_eq!(result.0, 0xFFFF_FFFF_FF00_FF00);
    }

    #[test]
    fn test_parallel_std_threads() {
        let pipeline = QuatPipeline::new().add_stage(QuatInvertStage);

        let mut chunks = vec![QuatChunk::new(0); 100];
        pipeline.execute_batch_parallel(&mut chunks, 4);

        for chunk in chunks {
            assert_eq!(chunk.0, u64::MAX);
        }
    }
}