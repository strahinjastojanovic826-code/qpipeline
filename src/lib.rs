pub mod builder;
pub mod chunk;
pub mod error;
pub mod stage;
pub mod ffi;
pub mod logic;
pub mod simd;

pub use builder::*;
pub use chunk::*;
pub use error::*;
pub use stage::*;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_quat_chunk_get_set_state() {
        let start = Instant::now();
        let mut chunk = QuatChunk::new(0);

        chunk.set_state(0, QuatState::Q3).unwrap();
        chunk.set_state(1, QuatState::Q1).unwrap();
        chunk.set_state(31, QuatState::Q2).unwrap();

        assert_eq!(chunk.get_state(0).unwrap(), QuatState::Q3);
        assert_eq!(chunk.get_state(1).unwrap(), QuatState::Q1);
        assert_eq!(chunk.get_state(31).unwrap(), QuatState::Q2);
        assert_eq!(chunk.get_state(2).unwrap(), QuatState::Q0);

        assert!(chunk.get_state(32).is_err());
        assert!(chunk.set_state(32, QuatState::Q1).is_err());
        
        println!("[BENCHMARK] test_quat_chunk_get_set_state took: {:?}", start.elapsed());
    }

    #[test]
    fn test_quat_chunk_count_and_invert() {
        let start = Instant::now();
        let chunk = QuatChunk::new(0);
        assert_eq!(chunk.count_state(QuatState::Q0), 32);
        assert_eq!(chunk.count_state(QuatState::Q3), 0);

        let inverted = chunk.invert();
        assert_eq!(inverted.count_state(QuatState::Q3), 32);
        assert_eq!(inverted.count_state(QuatState::Q0), 0);
        
        println!("[BENCHMARK] test_quat_chunk_count_and_invert took: {:?}", start.elapsed());
    }

    #[test]
    fn test_pipeline_sequential_execution() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);
        pipeline.push_stage(QuatXorMaskStage { mask: 0x0F0F_0F0F_0F0F_0F0F });

        let chunk = QuatChunk::new(0);
        let processed = pipeline.execute_chunk(chunk).unwrap();
        // Invertovanjem nule dobijamo sve jedinice (!0), a XOR sa maskom daje F0F0...
        assert_eq!(processed.raw(), 0xF0F0_F0F0_F0F0_F0F0);
        
        println!("[BENCHMARK] test_pipeline_sequential_execution took: {:?}", start.elapsed());
    }

    #[test]
    fn test_pipeline_parallel_smart_execution() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let mut chunks = vec![QuatChunk::new(0); 10_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        for chunk in chunks {
            assert_eq!(chunk.raw(), !0u64);
        }
        
        println!("[BENCHMARK] test_pipeline_parallel_smart_execution took: {:?}", start.elapsed());
    }

    #[test]
    fn test_ffi_lifecycle_and_execution() {
        let start = Instant::now();
        unsafe {
            let ptr = qpipeline_create();
            assert!(!ptr.is_null());

            assert_eq!(qpipeline_add_invert_stage(ptr), QuatFFIStatus::Ok);
            assert_eq!(qpipeline_add_xor_stage(ptr, 0xFF), QuatFFIStatus::Ok);

            let mut data = vec![0u64; 100];
            let status = qpipeline_execute_smart(ptr, data.as_mut_ptr(), data.len());
            assert_eq!(status, QuatFFIStatus::Ok);

            assert_eq!(data[0], !0xFFu64);

            qpipeline_free(ptr);
        }
        println!("[BENCHMARK] test_ffi_lifecycle_and_execution took: {:?}", start.elapsed());
    }

    #[test]
    fn test_ffi_null_pointer_safety() {
        let start = Instant::now();
        unsafe {
            assert_eq!(
                qpipeline_add_invert_stage(std::ptr::null_mut()),
                QuatFFIStatus::NullPointer
            );
            assert_eq!(
                qpipeline_execute_smart(std::ptr::null_mut(), std::ptr::null_mut(), 10),
                QuatFFIStatus::NullPointer
            );
        }
        println!("[BENCHMARK] test_ffi_null_pointer_safety took: {:?}", start.elapsed());
    }

    #[test]
    fn test_1_stress_massive_batch_execution() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);
        pipeline.push_stage(QuatXorMaskStage { mask: 0xAAAAAAAAAAAAAAAA });

        let count = 10_000_000;
        let mut chunks = vec![QuatChunk::new(0x5555555555555555); count];
        pipeline.execute_smart(&mut chunks).unwrap();

        assert_eq!(chunks.len(), count);
        assert_eq!(chunks[0].raw(), !0x5555555555555555 ^ 0xAAAAAAAAAAAAAAAA);
        assert_eq!(chunks[count - 1].raw(), !0x5555555555555555 ^ 0xAAAAAAAAAAAAAAAA);
        
        println!("[BENCHMARK] test_1_stress_massive_batch_execution took: {:?}", start.elapsed());
    }

    #[test]
    fn test_2_stress_thread_race_inspect_stage() {
        let start = Instant::now();
        let inspect_stage = QuatInspectStage::new(QuatState::Q3);
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);
        pipeline.push_stage(inspect_stage.clone());

        let mut chunks = vec![QuatChunk::new(0); 100_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        assert_eq!(inspect_stage.count(), 3_200_000);
        
        println!("[BENCHMARK] test_2_stress_thread_race_inspect_stage took: {:?}", start.elapsed());
    }

    #[test]
    fn test_3_stress_deep_pipeline_chaining() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        for i in 0..100 {
            pipeline.push_stage(QuatXorMaskStage { mask: i as u64 });
        }

        let mut chunks = vec![QuatChunk::new(123456789); 10_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        let mut expected_mask = 0u64;
        for i in 0..100 {
            expected_mask ^= i as u64;
        }

        assert_eq!(chunks[0].raw(), 123456789 ^ expected_mask);
        
        println!("[BENCHMARK] test_3_stress_deep_pipeline_chaining took: {:?}", start.elapsed());
    }

    #[test]
    fn test_4_stress_branchless_replacement_throughput() {
        let start = Instant::now();
        let mut chunk = QuatChunk::new(0);
        chunk.set_state(1, QuatState::Q1).unwrap();

        for _ in 0..1_000_000 {
            chunk.replace_state_branchless(QuatState::Q1, QuatState::Q2);
        }

        assert_eq!(chunk.get_state(1).unwrap(), QuatState::Q2);
        
        println!("[BENCHMARK] test_4_stress_branchless_replacement_throughput took: {:?}", start.elapsed());
    }

    #[test]
    fn test_5_boundary_single_element_parallel_fallback() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let mut chunks = vec![QuatChunk::new(0xFFFF)];
        pipeline.execute_batch_parallel(&mut chunks, 16).unwrap();

        assert_eq!(chunks[0].raw(), !0xFFFF);
        
        println!("[BENCHMARK] test_5_boundary_single_element_parallel_fallback took: {:?}", start.elapsed());
    }

    #[test]
    fn test_6_boundary_empty_slice() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let mut chunks: Vec<QuatChunk> = vec![];
        assert!(pipeline.execute_smart(&mut chunks).is_ok());
        assert!(pipeline.execute_batch_parallel(&mut chunks, 4).is_ok());
        
        println!("[BENCHMARK] test_6_boundary_empty_slice took: {:?}", start.elapsed());
    }

    #[test]
    fn test_7_stress_lazy_stream_processing() {
        let start = Instant::now();
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        // Svi čunkovi u stream-u moraju biti inicijalizovani sa 0 da bi nakon invertovanja bili !0
        let stream = (0..1_000_000).map(|_| QuatChunk::new(0));
        let processed_stream = pipeline.process_stream(stream);

        let count = processed_stream
            .filter_map(|res| res.ok())
            .filter(|chunk| chunk.raw() == !0)
            .count();

        assert_eq!(count, 1_000_000);
        
        println!("[BENCHMARK] test_7_stress_lazy_stream_processing took: {:?}", start.elapsed());
    }

    #[test]
    fn test_8_stress_ffi_rapid_creation_destruction() {
        let start = Instant::now();
        unsafe {
            for _ in 0..10_000 {
                let ptr = qpipeline_create();
                assert!(!ptr.is_null());
                qpipeline_add_invert_stage(ptr);
                qpipeline_free(ptr);
            }
        }
        println!("[BENCHMARK] test_8_stress_ffi_rapid_creation_destruction took: {:?}", start.elapsed());
    }

    #[test]
    fn test_9_stress_ffi_huge_array_pass() {
        let start = Instant::now();
        unsafe {
            let ptr = qpipeline_create();
            qpipeline_add_invert_stage(ptr);

            let mut raw_data = vec![0u64; 2_000_000];
            let status = qpipeline_execute_smart(ptr, raw_data.as_mut_ptr(), raw_data.len());

            assert_eq!(status, QuatFFIStatus::Ok);
            assert_eq!(raw_data[0], !0u64);
            assert_eq!(raw_data[1_999_999], !0u64);

            qpipeline_free(ptr);
        }
        println!("[BENCHMARK] test_9_stress_ffi_huge_array_pass took: {:?}", start.elapsed());
    }

    #[test]
    fn test_10_static_pipeline_zero_cost_fusion() {
        let start = Instant::now();
        let stage_a = QuatInvertStage;
        let stage_b = QuatXorMaskStage { mask: 0b1010 };
        let fused_stage = stage_a.then(stage_b);

        let chunk = QuatChunk::new(0b1111);
        let result = fused_stage.process(chunk).unwrap();

        assert_eq!(result.raw(), (!0b1111u64) ^ 0b1010);
        
        println!("[BENCHMARK] test_10_static_pipeline_zero_cost_fusion took: {:?}", start.elapsed());
    }

    #[test]
    fn test_quat_arithmetic() {
        let start = Instant::now();
        assert_eq!(QuatState::Q1.add(QuatState::Q2), QuatState::Q3);
        assert_eq!(QuatState::Q3.add(QuatState::Q1), QuatState::Q0);
        assert_eq!(QuatState::Q3.sub(QuatState::Q1), QuatState::Q2);
        assert_eq!(QuatState::Q0.sub(QuatState::Q1), QuatState::Q3);
        
        println!("[BENCHMARK] test_quat_arithmetic took: {:?}", start.elapsed());
    }

    #[test]
    fn test_quat_double_chunk_invert() {
        let start = Instant::now();
        let chunk = QuatDoubleChunk::new(0);
        let inverted = chunk.invert();
        assert_eq!(inverted.raw(), !0u128);
        
        println!("[BENCHMARK] test_quat_double_chunk_invert took: {:?}", start.elapsed());
    }

    #[test]
    fn test_quat_lookup_stage() {
        let start = Instant::now();
        let stage = QuatLookupStage::new([
            QuatState::Q3,
            QuatState::Q2,
            QuatState::Q1,
            QuatState::Q0,
        ]);

        let chunk = QuatChunk::new(0);
        let processed = stage.process(chunk).unwrap();

        let expected_val: u64 = !0u64;
        assert_eq!(processed.raw(), expected_val);
        
        println!("[BENCHMARK] test_quat_lookup_stage took: {:?}", start.elapsed());
    }

#[test]
fn test_extended_quat_logic_with_timing() {
    use std::time::Instant;
    use crate::logic::QuatLogicOps;

    let q1 = QuatState::Q1; // 0b01
    let q2 = QuatState::Q2; // 0b10

    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _ = q1.min(q2);
        let _ = q1.max(q2);
        let _ = q1.negate();
    }
    let duration = start.elapsed();
    println!("[BENCHMARK] 1,000,000 logic operations took: {:?}", duration);

    // Provera tačnosti
    assert_eq!(q1.min(q2), QuatState::Q1);
    assert_eq!(q1.max(q2), QuatState::Q2);
    assert_eq!(q1.negate(), QuatState::Q2);
}

#[test]
fn test_quat_truth_table_with_timing() {
    use std::time::Instant;
    use crate::logic::QuatTruthTable;

    let table = [[QuatState::Q0; 4]; 4];
    let tt = QuatTruthTable::new(table);

    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _ = tt.evaluate(QuatState::Q1, QuatState::Q2);
    }
    let duration = start.elapsed();
    println!("[BENCHMARK] 1,000,000 truth table evaluations took: {:?}", duration);

    assert_eq!(tt.evaluate(QuatState::Q1, QuatState::Q2), QuatState::Q0);
}

#[test]
fn test_quat_matrix_with_timing() {
    use std::time::Instant;
    use crate::logic::QuatMatrix;

    let start = Instant::now();
    let mut matrix = QuatMatrix::new(100, 100, QuatState::Q0);
    matrix.set(50, 50, QuatState::Q3);
    let _transposed = matrix.transpose();
    let duration = start.elapsed();
    
    println!("[BENCHMARK] Matrix 100x100 allocation, set and transpose took: {:?}", duration);
    assert_eq!(matrix.get(50, 50), Some(QuatState::Q3));
}

#[test]
fn test_simd_engine_with_timing() {
    use std::time::Instant;
    use crate::simd::SimdEngine;
    use crate::chunk::QuatChunk;

    let mut chunks = vec![QuatChunk::new(0x01010101); 10_000];

    let start = Instant::now();
    SimdEngine::invert_portable(&mut chunks);
    let duration = start.elapsed();

    println!("[BENCHMARK] SIMD/Portable invert over 10,000 chunks took: {:?}", duration);
    assert_eq!(chunks[0].raw(), !0x01010101);
}

}