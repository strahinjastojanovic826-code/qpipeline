pub mod builder;
pub mod chunk;
pub mod error;
pub mod stage;
pub mod ffi;

pub use builder::*;
pub use chunk::*;
pub use error::*;
pub use stage::*;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quat_chunk_get_set_state() {
        let mut chunk = QuatChunk::new(0);

        // Postavljanje stanja na određenim pozicijama
        chunk.set_state(0, QuatState::Q3).unwrap();
        chunk.set_state(1, QuatState::Q1).unwrap();
        chunk.set_state(31, QuatState::Q2).unwrap();

        // Provera dohvaćenih stanja
        assert_eq!(chunk.get_state(0).unwrap(), QuatState::Q3);
        assert_eq!(chunk.get_state(1).unwrap(), QuatState::Q1);
        assert_eq!(chunk.get_state(31).unwrap(), QuatState::Q2);
        assert_eq!(chunk.get_state(2).unwrap(), QuatState::Q0);

        // Provera greške za indeks van opsega
        assert!(chunk.get_state(32).is_err());
        assert!(chunk.set_state(32, QuatState::Q1).is_err());
    }

    #[test]
    fn test_quat_chunk_count_and_invert() {
        // Blok sa 32 x Q0 (svi bitovi 0)
        let chunk = QuatChunk::new(0);
        assert_eq!(chunk.count_state(QuatState::Q0), 32);
        assert_eq!(chunk.count_state(QuatState::Q3), 0);

        // Invertovanjem 0 nastaje !0 (svi bitovi 1 -> Q3)
        let inverted = chunk.invert();
        assert_eq!(inverted.count_state(QuatState::Q3), 32);
        assert_eq!(inverted.count_state(QuatState::Q0), 0);
    }

    #[test]
    fn test_pipeline_sequential_execution() {
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);
        pipeline.push_stage(QuatXorMaskStage { mask: 0x0F0F_0F0F_0F0F_0F0F });

        let chunk = QuatChunk::new(0);
        let processed = pipeline.execute_chunk(chunk).unwrap();

        // !0 ^ 0x0F... = !0x0F...
        assert_eq!(processed.raw(), !0x0F0F_0F0F_0F0F_0F0F);
    }

    #[test]
    fn test_pipeline_parallel_smart_execution() {
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        // Kreiranje većeg niza blokova za aktiviranje paralelne obrade
        let mut chunks = vec![QuatChunk::new(0); 10_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        for chunk in chunks {
            assert_eq!(chunk.raw(), !0u64);
        }
    }

    #[test]
    fn test_ffi_lifecycle_and_execution() {
        unsafe {
            // Kreiranje cjevovoda preko FFI
            let ptr = qpipeline_create();
            assert!(!ptr.is_null());

            // Dodavanje faza
            assert_eq!(qpipeline_add_invert_stage(ptr), QuatFFIStatus::Ok);
            assert_eq!(qpipeline_add_xor_stage(ptr, 0xFF), QuatFFIStatus::Ok);

            // Izvršavanje nad nizom podataka
            let mut data = vec![0u64; 100];
            let status = qpipeline_execute_smart(ptr, data.as_mut_ptr(), data.len());
            assert_eq!(status, QuatFFIStatus::Ok);

            // Provera da li su podaci izmenjeni
            assert_eq!(data[0], !0xFFu64);

            // Oslobađanje memorije
            qpipeline_free(ptr);
        }
    }

    #[test]
    fn test_ffi_null_pointer_safety() {
        unsafe {
            // Provera da li FFI funkcije bezbedno vraćaju NullPointer status
            assert_eq!(
                qpipeline_add_invert_stage(std::ptr::null_mut()),
                QuatFFIStatus::NullPointer
            );
            assert_eq!(
                qpipeline_execute_smart(std::ptr::null(), std::ptr::null_mut(), 10),
                QuatFFIStatus::NullPointer
            );
        }
    }
    #[test]
    fn test_1_stress_massive_batch_execution() {
        // Stres test: Obrada 10 miliona blokova (320 miliona kvartarnih stanja)
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);
        pipeline.push_stage(QuatXorMaskStage { mask: 0xAAAAAAAAAAAAAAAA });

        let count = 10_000_000;
        let mut chunks = vec![QuatChunk::new(0x5555555555555555); count];

        pipeline.execute_smart(&mut chunks).unwrap();

        // Evaluacija tačnosti svih 10 miliona elemenata
        assert_eq!(chunks.len(), count);
        assert_eq!(chunks[0].raw(), !0x5555555555555555 ^ 0xAAAAAAAAAAAAAAAA);
        assert_eq!(chunks[count - 1].raw(), !0x5555555555555555 ^ 0xAAAAAAAAAAAAAAAA);
    }

    #[test]
    fn test_2_stress_thread_race_inspect_stage() {
        // Stres test konkurentnosti: 100 hiljada blokova istovremeno ažurira Atomic brojač
        let inspect_stage = QuatInspectStage::new(QuatState::Q3);
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage); // Prevara 0 u !0 (svi postaju Q3)
        pipeline.push_stage(inspect_stage.clone());

        let mut chunks = vec![QuatChunk::new(0); 100_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        // 100,000 chunks * 32 stanja po chunk-u = 3,200,000 izbrojanih Q3 stanja
        assert_eq!(inspect_stage.count(), 3_200_000);
    }

    #[test]
    fn test_3_stress_deep_pipeline_chaining() {
        // Stres test dubine cjevovoda: Lanac od 100 faza
        let mut pipeline = QuatPipeline::new();
        for i in 0..100 {
            pipeline.push_stage(QuatXorMaskStage { mask: i as u64 });
        }

        let mut chunks = vec![QuatChunk::new(123456789); 10_000];
        pipeline.execute_smart(&mut chunks).unwrap();

        // Izračunaj očekivanu masku
        let mut expected_mask = 0u64;
        for i in 0..100 {
            expected_mask ^= i as u64;
        }

        assert_eq!(chunks[0].raw(), 123456789 ^ expected_mask);
    }

    #[test]
fn test_4_stress_branchless_replacement_throughput() {
    let mut chunk = QuatChunk::new(0);
    chunk.set_state(1, QuatState::Q1).unwrap();

    for _ in 0..1_000_000 {
        chunk.replace_state_branchless(QuatState::Q1, QuatState::Q2);
    }

    // Stanje Q1 je moralo postati Q2
    assert_eq!(chunk.get_state(1).unwrap(), QuatState::Q2);
}

    #[test]
    fn test_5_boundary_single_element_parallel_fallback() {
        // Test graničnog slučaja: Paralelno izvršavanje nad samo 1 elementom (mora pasti na sekvencijalno)
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let mut chunks = vec![QuatChunk::new(0xFFFF)];
        pipeline.execute_batch_parallel(&mut chunks, 16).unwrap();

        assert_eq!(chunks[0].raw(), !0xFFFFu64);
    }

    #[test]
    fn test_6_boundary_empty_slice() {
        // Test praznog niza: Ne smije uzrokovati paniku ili gubljenje memorije
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let mut chunks: Vec<QuatChunk> = vec![];
        assert!(pipeline.execute_smart(&mut chunks).is_ok());
        assert!(pipeline.execute_batch_parallel(&mut chunks, 4).is_ok());
    }

    #[test]
    fn test_7_stress_lazy_stream_processing() {
        // Stres test za lazy iterator (streaming obrada bez alokacije čitavog vektora)
        let mut pipeline = QuatPipeline::new();
        pipeline.push_stage(QuatInvertStage);

        let stream = (0..1_000_000).map(QuatChunk::new);
        let processed_stream = pipeline.process_stream(stream);

        let count = processed_stream
            .filter_map(|res| res.ok())
            .filter(|chunk| chunk.raw() == !0)
            .count();

        // Samo prvi (0) kada se invertuje daje !0
        assert_eq!(count, 1);
    }

    #[test]
    fn test_8_stress_ffi_rapid_creation_destruction() {
        // Stres test za FFI: Brzo kreiranje i uništavanje 10,000 cjevovoda u memoriji
        unsafe {
            for _ in 0..10_000 {
                let ptr = qpipeline_create();
                assert!(!ptr.is_null());
                qpipeline_add_invert_stage(ptr);
                qpipeline_free(ptr);
            }
        }
    }

    #[test]
    fn test_9_stress_ffi_huge_array_pass() {
        // Stres test FFI granice: Slanje 2 miliona raw u64 elemenata preko C interfejsa
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
    }

    #[test]
    fn test_10_static_pipeline_zero_cost_fusion() {
        // Test static pipeline spajanja bez alokacija na hipu
        let stage_a = QuatInvertStage;
        let stage_b = QuatXorMaskStage { mask: 0b1010 };
        let fused_stage = stage_a.then(stage_b);

        let chunk = QuatChunk::new(0b1111);
        let result = fused_stage.process(chunk).unwrap();

        assert_eq!(result.raw(), (!0b1111u64) ^ 0b1010);
    }
    #[test]
fn test_quat_arithmetic() {
    // Sabiranje po modulu 4
    assert_eq!(QuatState::Q1.add(QuatState::Q2), QuatState::Q3);
    assert_eq!(QuatState::Q3.add(QuatState::Q1), QuatState::Q0); // Prelivanje (overflow) na Q0
    
    // Oduzimanje po modulu 4
    assert_eq!(QuatState::Q3.sub(QuatState::Q1), QuatState::Q2);
    assert_eq!(QuatState::Q0.sub(QuatState::Q1), QuatState::Q3); // Podlivanje (underflow) na Q3
}
#[test]
fn test_quat_double_chunk_invert() {
    let chunk = QuatDoubleChunk::new(0);
    let inverted = chunk.invert();
    assert_eq!(inverted.raw(), !0u128);
}

#[test]
fn test_quat_lookup_stage() {
    // Invertuje svako stanje: Q0->Q3, Q1->Q2, Q2->Q1, Q3->Q0
    let stage = QuatLookupStage::new([
        QuatState::Q3,
        QuatState::Q2,
        QuatState::Q1,
        QuatState::Q0,
    ]);

    let chunk = QuatChunk::new(0); // Sva stanja su Q0 (0b00)
    let processed = stage.process(chunk).unwrap();

    // Proveravamo da li je Q0 prešao u Q3 (0b11 za svaki par bitova -> svi bitovi 1)
    let expected_val: u64 = !0u64; // Ili 0xFFFFFFFFFFFFFFFF
    assert_eq!(processed.raw(), expected_val);
}

}