use crate::chunk::QuatChunk;
use crate::stage::QuatStage;
use std::sync::Arc;
use std::thread;

pub struct QuatPipeline {
    stages: Vec<Arc<dyn QuatStage>>,
}

impl QuatPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage<S: QuatStage + 'static>(mut self, stage: S) -> Self {
        self.stages.push(Arc::new(stage));
        self
    }

    /// Sekvencijalno izvršavanje jednog bloka
    #[inline]
    pub fn execute_chunk(&self, mut chunk: QuatChunk) -> QuatChunk {
        for stage in &self.stages {
            chunk = stage.process(chunk);
        }
        chunk
    }

    /// Sekvencijalno izvršavanje nad celim nizom
    pub fn execute_batch(&self, chunks: &mut [QuatChunk]) {
        for chunk in chunks.iter_mut() {
            *chunk = self.execute_chunk(*chunk);
        }
    }

    /// Paralelno izvršavanje pomoću std::thread (bez eksternih biblioteka)
    pub fn execute_batch_parallel(&self, chunks: &mut [QuatChunk], num_threads: usize) {
        if num_threads <= 1 || chunks.is_empty() {
            self.execute_batch(chunks);
            return;
        }

        let chunk_size = (chunks.len() + num_threads - 1) / num_threads;
        let stages = self.stages.clone();

        thread::scope(|s| {
            for chunk_slice in chunks.chunks_mut(chunk_size) {
                let stages = stages.clone();
                s.spawn(move || {
                    for chunk in chunk_slice.iter_mut() {
                        let mut val = *chunk;
                        for stage in &stages {
                            val = stage.process(val);
                        }
                        *chunk = val;
                    }
                });
            }
        });
    }
}