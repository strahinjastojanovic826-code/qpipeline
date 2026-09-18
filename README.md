# qpipeline

`qpipeline` is a dependency-free, high-performance processing engine designed for **4-state (2-bit logic)** computing architectures. Built specifically to complement the 2-bit quat ecosystem (`Q0`, `Q1`, `Q2`, `Q3`), it focuses on explicit bit-packing, zero-cost abstractions, and maximum CPU cache efficiency.

---

## Features

* **Zero External Dependencies** — Built using pure Rust and `std` primitives.
* **Compact Bit-Packing** — Operates on 64-bit blocks (`QuatChunk`) holding 32 packed quat states simultaneously.
* **Trait-Based Pipeline System** — Easily construct modular data transformations via the `QuatStage` trait.
* **In-Memory Parallel Execution** — Utilizes Rust's native `std::thread::scope` for high-throughput batch processing without runtime overhead.
* **Predictable & Deterministic** — Designed for specialized low-level computing, memory-constrained environments, and systems programming.

---

## Data Model

In `qpipeline`, 4-state logic is mapped across 2-bit representations:

| State | Bit Pattern | Value |
| :--- | :--- | :--- |
| `Q0` | `0b00` | `0` |
| `Q1` | `0b01` | `1` |
| `Q2` | `0b10` | `2` |
| `Q3` | `0b11` | `3` |

A single `QuatChunk` encapsulates 32 of these states into a standard `u64` primitive, minimizing memory footprint and drastically reducing cache misses during execution.

---

## Quickstart

Add `qpipeline` to your project's `Cargo.toml`:

```toml
[dependencies]
qpipeline = "0.1.0"

---

Basic Pipeline Setup

```rust
use qpipeline::{QuatPipeline, QuatChunk, QuatInvertStage, QuatXorMaskStage};

fn main() {
    // 1. Build a pipeline with chainable stages
    let pipeline = QuatPipeline::new()
        .add_stage(QuatInvertStage)
        .add_stage(QuatXorMaskStage { mask: 0x00FF_00FF });

    // 2. Initialize a packed 64-bit chunk (32 quats initialized to Q0)
    let chunk = QuatChunk::new(0x0000_0000);

    // 3. Process the chunk through the pipeline
    let result = pipeline.execute_chunk(chunk);

    println!("Resulting raw chunk: 0x{:X}", result.0);
}

---

Batch & Parallel Processing

For handling large datasets, qpipeline offers both sequential and multi-threaded scope execution:

use qpipeline::{QuatPipeline, QuatChunk, QuatInvertStage};

```rust
fn main() {
    let pipeline = QuatPipeline::new().add_stage(QuatInvertStage);

    // Create a vector of 10,000 packed chunks (320,000 individual quat states)
    let mut batch = vec![QuatChunk::new(0); 10_000];

    // Parallel processing across 8 OS native threads (No external crates required)
    pipeline.execute_batch_parallel(&mut batch, 8);
}

---

Custom Stages

Implement the QuatStage trait to introduce your own 4-state transformations:

```rust
use qpipeline::{QuatStage, QuatChunk};

pub struct CustomMaskStage {
    pub pattern: u64,
}

impl QuatStage for CustomMaskStage {
    #[inline(always)]
    fn process(&self, chunk: QuatChunk) -> QuatChunk {
        QuatChunk(chunk.0 & self.pattern)
    }
}

---