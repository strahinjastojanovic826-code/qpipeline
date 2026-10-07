use crate::chunk::QuatChunk;
use crate::error::PipelineError;
use crate::stage::QuatStage;
use std::sync::Arc;
use std::thread;

/// Glavni dinamički cjevovod za obradu kvartarnih blokova.
#[derive(Default)]
pub struct QuatPipeline {
    stages: Vec<Arc<dyn QuatStage>>,
}

impl QuatPipeline {
    #[inline]
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Dodaje novu fazu u cjevovod (prati Fluent Builder pattern).
    pub fn add_stage<S: QuatStage + 'static>(mut self, stage: S) -> Self {
        self.stages.push(Arc::new(stage));
        self
    }

    /// Dodaje novu fazu putem referenci (ako se pipeline ne konzumira).
    pub fn push_stage<S: QuatStage + 'static>(&mut self, stage: S) {
        self.stages.push(Arc::new(stage));
    }

    /// Sekvencijalna obrada jednog bloka kroz sve faze.
    #[inline]
    pub fn execute_chunk(&self, mut chunk: QuatChunk) -> Result<QuatChunk, PipelineError> {
        for stage in &self.stages {
            chunk = stage.process(chunk)?;
        }
        Ok(chunk)
    }

    /// Sekvencijalna obrada niza blokova na licu mesta (in-place).
    pub fn execute_batch(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        for chunk in chunks.iter_mut() {
            *chunk = self.execute_chunk(*chunk)?;
        }
        Ok(())
    }

    /// Pametno raspoređivanje: automatski bira sekvencijalno ili paralelno
    /// izvršavanje u zavisnosti od broja dostupnih jezgara i veličine podataka.
    pub fn execute_smart(&self, chunks: &mut [QuatChunk]) -> Result<(), PipelineError> {
        const MIN_CHUNKS_PER_THREAD: usize = 2048;

        let available_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        if chunks.len() < MIN_CHUNKS_PER_THREAD || available_threads <= 1 {
            self.execute_batch(chunks)
        } else {
            let optimal_threads = (chunks.len() / MIN_CHUNKS_PER_THREAD).min(available_threads);
            self.execute_batch_parallel(chunks, optimal_threads)
        }
    }

    /// Paralelna obrada pomoću `thread::scope` bez alokacija na hipu i bez lekovitih overhead-a.
    pub fn execute_batch_parallel(
        &self,
        chunks: &mut [QuatChunk],
        num_threads: usize,
    ) -> Result<(), PipelineError> {
        if num_threads <= 1 || chunks.is_empty() {
            return self.execute_batch(chunks);
        }

        let chunk_size = (chunks.len() + num_threads - 1) / num_threads;

        thread::scope(|s| {
            let mut handles = Vec::with_capacity(num_threads);

            for chunk_slice in chunks.chunks_mut(chunk_size) {
                let handle = s.spawn(move || -> Result<(), PipelineError> {
                    for chunk in chunk_slice.iter_mut() {
                        let mut val = *chunk;
                        for stage in &self.stages {
                            val = stage.process(val)?;
                        }
                        *chunk = val;
                    }
                    Ok(())
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap()?
            }

            Ok(())
        })
    }

    /// Lenja obrada preko iteratora (Lazy Streaming).
    pub fn process_stream<'a>(
        &'a self,
        iter: impl Iterator<Item = QuatChunk> + 'a,
    ) -> impl Iterator<Item = Result<QuatChunk, PipelineError>> + 'a {
        iter.map(move |chunk| self.execute_chunk(chunk))
    }
}