use crate::chunk::QuatChunk;
use crate::error::PipelineError;
use crate::stage::QuatStage;
use std::sync::Arc;
use std::thread;
use crate::QuatBlock;

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

    pub fn execute_simd(&self, chunks: &mut [QuatChunk]) {
    #[cfg(target_arch = "x86_64")]
    {
        crate::simd::SimdEngine::invert_avx2(chunks);
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        crate::simd::SimdEngine::invert_portable(chunks);
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PipelineTelemetry {
    pub processed_chunks: usize,
    pub threads_used: usize,
    pub elapsed_nanos: u128,
}

pub trait GenericStage<B: QuatBlock>: Send + Sync {
    fn process(&self, block: B) -> Result<B, PipelineError>;
}

pub struct GenericQuatPipeline<B: QuatBlock> {
    stages: Vec<Arc<dyn GenericStage<B>>>,
}

impl<B: QuatBlock> Default for GenericQuatPipeline<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: QuatBlock> GenericQuatPipeline<B> {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage<S: GenericStage<B> + 'static>(mut self, stage: S) -> Self {
        self.stages.push(Arc::new(stage));
        self
    }

    pub fn execute_block(&self, mut block: B) -> Result<B, PipelineError> {
        for stage in &self.stages {
            block = stage.process(block)?;
        }
        Ok(block)
    }

    pub fn execute_slice(&self, blocks: &mut [B]) -> Result<(), PipelineError> {
        for block in blocks.iter_mut() {
            *block = self.execute_block(*block)?;
        }
        Ok(())
    }
}

impl QuatPipeline {
    /// Executes with telemetry and converts worker thread panics into PipelineError
    pub fn execute_smart_with_telemetry(
        &self,
        chunks: &mut [QuatChunk],
        min_chunks_per_thread: usize,
    ) -> Result<PipelineTelemetry, PipelineError> {
        let start = std::time::Instant::now();
        let min_per_thread = min_chunks_per_thread.max(1);

        let available_threads: usize = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let threads_used = if chunks.len() < min_per_thread || available_threads <= 1 {
            self.execute_batch(chunks)?;
            1
        } else {
            let optimal_threads = (chunks.len() / min_per_thread)
                .min(available_threads)
                .max(1);

            if optimal_threads <= 1 || chunks.is_empty() {
                self.execute_batch(chunks)?;
                1
            } else {
                let chunk_size = (chunks.len() + optimal_threads - 1) / optimal_threads;
                let mut thread_err: Option<PipelineError> = None;

                thread::scope(|s| {
                    let mut handles = Vec::with_capacity(optimal_threads);
                    for chunk_slice in chunks.chunks_mut(chunk_size) {
                        let handle = s.spawn(move || -> Result<(), PipelineError> {
                            for chunk in chunk_slice.iter_mut() {
                                *chunk = self.execute_chunk(*chunk)?;
                            }
                            Ok(())
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        match handle.join() {
                            Ok(Ok(())) => {}
                            Ok(Err(e)) => {
                                if thread_err.is_none() {
                                    thread_err = Some(e);
                                }
                            }
                            Err(_) => {
                                if thread_err.is_none() {
                                    thread_err = Some(PipelineError::Custom(
                                        "Worker thread panicked during pipeline execution".to_string(),
                                    ));
                                }
                            }
                        }
                    }
                });

                if let Some(err) = thread_err {
                    return Err(err);
                }
                optimal_threads
            }
        };

        Ok(PipelineTelemetry {
            processed_chunks: chunks.len(),
            threads_used,
            elapsed_nanos: start.elapsed().as_nanos(),
        })
    }
}