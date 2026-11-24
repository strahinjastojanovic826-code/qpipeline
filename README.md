# QuatPipeline

`qpipeline` is a high-performance, thread-safe Rust library designed for fast, multi-threaded batch processing of quaternary data representations (`QuatChunk`). It includes zero-copy parallel processing pipelines, branchless bitwise transformations, and production-ready FFI (Foreign Function Interface) bindings for seamless integration with C, C++, and C#.

---

## Features

- **High-Performance Quaternary Data Operations**: Efficient bitwise manipulation of packed quaternary states.
- **Flexible Pipeline System**: Chain multiple execution stages (`QuatInvertStage`, `QuatXorMaskStage`, etc.) sequentially or staticaly.
- **Smart Parallel Processing**: Automatically scales workload across multiple CPU threads using `std::thread::scope` without runtime allocation overhead.
- **FFI Bindings for C, C++, and C#**: Safe, C-compatible binary interface with full panic boundaries (`catch_unwind`) to prevent crashes across foreign ABI boundaries.
- **Zero-Copy Architecture**: Process memory in-place directly over raw pointers or native slices.
- **Extended Quaternary Arithmetic**: Built-in support for modular arithmetic (addition and subtraction modulo 4) directly on quaternary states (`QuatState`).
- **SIMD / Extended Vectorization (`QuatDoubleChunk`)**: High-performance 128-bit blocks capable of processing 64 quaternary states within a single CPU word.
- **Serde Serialization Support**: Optional features for serializing and deserializing chunks and states for easy data persistence and networking.
- **Advanced Lookup Table (LUT) Stages**: Flexible custom transformation stages (`QuatLookupStage`) using user-defined mapping matrices.
- **Extended Quaternary Logic**: Built-in support for advanced Łukasiewicz logical operators (`min`, `max`, `negate`).
- **Quaternary Matrices & Truth Tables**: 2D quaternary matrices with transpose support and arbitrary 4x4 truth table evaluators.
- **Lazy Stream Processing**: Iterator-based streaming transformations without allocating intermediate vectors.
- **Batch Stage Execution (`process_batch`)**: Overridable batch hooks on stages for faster slice and vectorised processing.
- **Generic Block Pipeline (`QuatBlock` & `GenericQuatPipeline`)**: Generic block abstraction supporting both `QuatChunk` and `QuatDoubleChunk` without duplicating pipeline logic.
- **Matrix Algebra Operations (`QuatMatrix`)**: Quaternary matrix multiplication using `(max, min)` semiring and modular (`mod 4`) arithmetic with dimension validation.
- **Execution Telemetry & Panic Safety**: Execution timing and thread metrics (`PipelineTelemetry`) with safe worker thread error/panic propagation.

---

## Architecture Overview

| Module | Description |
| :--- | :--- |
| `builder` | Pipeline creation and execution engine (`QuatPipeline`). |
| `chunk` | Core data representation (`QuatChunk`) and quaternary bitwise operations. |
| `stage` | Individual processing logic and transformation stages. |
| `ffi` | C-compatible API layer for external languages (C, C++, C#). |
| `error` | Domain-specific pipeline error types. |

---

## Benchmark

| Test / Benchmark Case | Category | Measured Time | Workload / Scope | Status |
| :--- | :--- | :--- | :--- | :--- |
| `test_quat_arithmetic` | Arithmetic | `100ns` | Modular `add` & `sub` (`mod 4`) operations | `ok` |
| `test_quat_double_chunk_invert` | Block / Bitwise | `100ns` | 128-bit `QuatDoubleChunk` (64 states) invert | `ok` |
| `test_ffi_null_pointer_safety` | FFI Safety | `200ns` | Null pointer checks on FFI stage add & execute | `ok` |
| `test_quat_chunk_count_and_invert` | Block / SWAR | `200ns` | `count_state` and `invert` on `QuatChunk` | `ok` |
| `test_quat_chunk_get_set_state` | Block / Indexing | `800ns` | Bounds checks & `get_state` / `set_state` on `QuatChunk` | `ok` |
| `test_quat_lookup_stage` | Stage / LUT | `2.6µs` | `QuatLookupStage` state mapping | `ok` |
| `test_6_boundary_empty_slice` | Boundary | `4.2µs` | Empty slice on `execute_smart` & `execute_batch_parallel` | `ok` |
| `test_pipeline_sequential_execution` | Pipeline | `5.2µs` | Sequential `QuatInvertStage` + `QuatXorMaskStage` | `ok` |
| `test_10_static_pipeline_zero_cost_fusion` | Static Pipeline | `7.7µs` | Compile-time fused `StaticPipeline` execution | `ok` |
| `test_ffi_lifecycle_and_execution` | FFI | `20.1µs` | Create, add stages, execute 100 `u64` chunks, free | `ok` |
| `test_simd_engine_with_timing` | SIMD / Batch | `57.2µs` | `SimdEngine::invert_portable` over `10,000` chunks | `ok` |
| `test_stage_process_batch_with_timing` | Batch Stage | `367.3µs` | `process_batch` across 3 stages on `10,000` chunks | `ok` |
| `test_quat_matrix_with_timing` | Matrix | `485µs` | `100x100` `QuatMatrix` allocation, `set`, and `transpose` | `ok` |
| `test_pipeline_parallel_smart_execution` | Parallel Pipeline | `807µs` | `execute_smart` over `10,000` chunks | `ok` |
| `test_generic_block_and_telemetry_pipeline_with_timing` | Generic & Telemetry | `964µs` | `GenericQuatPipeline` + `execute_smart_with_telemetry` | `ok` |
| `test_quat_matrix_multiplication_with_timing` | Matrix Algebra | `3.8817ms` | `32x32` `mul_min_max` + `mul_modular` + error checks | `ok` |
| `test_8_stress_ffi_rapid_creation_destruction` | Stress / FFI | `6.4843ms` | `10,000` create/add-stage/free FFI cycles | `ok` |
| `test_5_boundary_single_element_parallel_fallback` | Boundary / Parallel | `8.9076ms` | Single-element slice with `num_threads: 16` | `ok` |
| `test_quat_truth_table_with_timing` | Truth Table | `9.4372ms` | `1,000,000` `QuatTruthTable::evaluate` calls | `ok` |
| `test_2_stress_thread_race_inspect_stage` | Stress / Concurrency | `10.7275ms` | Atomic `QuatInspectStage` across `100,000` chunks | `ok` |
| `test_3_stress_deep_pipeline_chaining` | Stress / Pipeline | `13.078ms` | `100` chained XOR stages over `10,000` chunks | `ok` |
| `test_extended_quat_logic_with_timing` | Logic | `14.4251ms` | `1,000,000` `min`, `max`, and `negate` iterations | `ok` |
| `test_9_stress_ffi_huge_array_pass` | Stress / FFI | `14.8513ms` | `2,000,000` `u64` elements via `qpipeline_execute_smart` | `ok` |
| `test_extended_multivalued_logic_and_from_fn_with_timing` | Multivalued Logic | `32.9382ms` | `1,000,000` Łukasiewicz, Gödel, consensus & `from_fn` | `ok` |
| `test_7_stress_lazy_stream_processing` | Stress / Streaming | `66.691ms` | `1,000,000` chunks via `process_stream` iterator | `ok` |
| `test_1_stress_massive_batch_execution` | Stress / Batch | `120.0219ms` | `10,000,000` chunks through invert + XOR pipeline | `ok` |
| `test_4_stress_branchless_replacement_throughput` | Stress / Branchless | `335.7845ms` | `1,000,000` `replace_state_branchless` iterations | `ok` |

---

# Third-Party Licenses

This software includes open-source components developed by third parties. Below are the corresponding license notices and full legal texts for each component used in this project.

## 1. Serde

* **Author:** David Tolnay
* **Repository:** https://github.com/serde-rs/serde
* **License:** MIT / Apache 2.0 (Dual License)

### MIT License

Copyright (c) 2017 David Tolnay

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

### Apache License, Version 2.0

Copyright (c) 2017 David Tolnay

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

"License" shall mean the terms and conditions for use, reproduction, and distribution as defined by Sections 1 through 9 of this document.

"Licensor" shall mean the copyright owner or entity authorized by the copyright owner that is granting the License.

"Legal Entity" shall mean the union of the acting entity and all other entities that control, are controlled by, or are under common control with that entity. For the purposes of this definition, "control" means (i) the power, direct or indirect, to cause the direction or management of such entity, whether by contract or otherwise, or (ii) ownership of fifty percent (50%) or more of the outstanding shares, or (iii) beneficial ownership of such entity.

"You" (or "Your") shall mean an individual or Legal Entity exercising permissions granted by this License.

"Source" form shall mean the preferred form for making modifications, including but not limited to software source code, documentation source, and configuration files.

"Object" form shall mean any form resulting from mechanical transformation or translation of a Source form, including but not limited to compiled object code, generated documentation, and conversions to other media types.

"Work" shall mean the work of authorship, whether in Source or Object form, made available under the License, as indicated by a copyright notice that is included in or attached to the work (an example is provided in the Appendix below).

"Derivative Works" shall mean any work, whether in Source or Object form, that is based on (or derived from) the Work and for which the editorial revisions, annotations, elaborations, or other modifications represent, as a whole, an original work of authorship. For the purposes of this License, Derivative Works shall not include works that remain separable from, or merely link (or bind by name) to the interfaces of, the Work and Derivative Works thereof.

"Contribution" shall mean any work of authorship, including the original version of the Work and any modifications or additions to that Work or Derivative Works thereof, that is intentionally submitted to Licensor for inclusion in the Work by the copyright owner or by an individual or Legal Entity authorized to submit on behalf of the copyright owner. For the purposes of this definition, "submitted" means any form of electronic, verbal, or written communication sent to the Licensor or its representatives, including but not limited to communication on electronic mailing lists, source code control systems, and issue tracking systems that are managed by, or on behalf of, the Licensor for the purpose of discussing and improving the Work, but excluding communication that is conspicuously marked or otherwise designated in writing by the copyright owner as "Not a Contribution."

"Contributor" shall mean Licensor and any individual or Legal Entity on behalf of whom a Contribution has been received by Licensor and subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of this License, each Contributor hereby grants to You a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable copyright license to reproduce, prepare Derivative Works of, publicly display, publicly perform, sublicense, and distribute the Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of this License, each Contributor hereby grants to You a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable (except as stated in this section) patent license to make, have made, use, offer to sell, sell, import, and otherwise transfer the Work, where such license applies only to those patent claims licensable by such Contributor that are necessarily infringed by their Contribution(s) alone or by combination of their Contribution(s) with the Work to which such Contribution(s) was submitted. If You institute patent litigation against any entity (including a cross-claim or counterclaim in a lawsuit) alleging that the Work or a Contribution incorporated within the Work constitutes direct or contributory patent infringement, then any patent licenses granted to You under this License for that Work shall terminate as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the Work or Derivative Works thereof in any medium, with or without modifications, and in Source or Object form, provided that You meet the following conditions:

   a. You must give any other recipients of the Work or Derivative Works a copy of this License; and

   b. You must cause any modified files to carry prominent notices stating that You changed the files; and

   c. You must retain, in the Source form of any Derivative Works that You distribute, all copyright, patent, trademark, and attribution notices from the Source form of the Work, excluding those notices that do not pertain to any part of the Derivative Works; and

   d. If the Work includes a "NOTICE" text file as part of its distribution, then any Derivative Works that You distribute must include a readable copy of the attribution notices contained within such NOTICE file, excluding those notices that do not pertain to any part of the Derivative Works, in at least one of the following places: within a NOTICE text file distributed as part of the Derivative Works; within the Source form or documentation, if provided along with the Derivative Works; or, within a display generated by the Derivative Works, if and wherever such third-party notices normally appear. The contents of the NOTICE file are for informational purposes only and do not modify the License. You may add Your own attribution notices within Derivative Works that You distribute, alongside or as an addendum to the NOTICE text from the Work, provided that such additional attribution notices cannot be construed as modifying the License.

You may add Your own copyright statement to Your modifications and may provide additional or different license terms and conditions for use, reproduction, or distribution of Your modifications, or for any such Derivative Works as a whole, provided Your use, reproduction, and distribution of the Work otherwise complies with the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise, any Contribution intentionally submitted for inclusion in the Work by You to the Licensor shall be under the terms and conditions of this License, without any additional terms or conditions. Notwithstanding the above, nothing herein shall supersede or modify the terms of any separate license agreement you may have executed with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade names, trademarks, service marks, or product names of the Licensor, except as required for reasonable and customary use in describing the origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or agreed to in writing, Licensor provides the Work (and each Contributor provides its Contributions) on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied, including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE. You are solely responsible for determining the appropriateness of using or redistributing the Work and assume any risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory, whether in tort (including negligence), contract, or otherwise, unless required by applicable law (such as deliberate and grossly negligent acts) or agreed to in writing, shall any Contributor be liable to You for damages, including any direct, indirect, special, incidental, or consequential damages of any character arising as a result of this License or out of the use or inability to use the Work (including but not limited to damages for loss of goodwill, work stoppage, computer failure or malfunction, or any and all other commercial damages or losses), even if such Contributor has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing the Work or Derivative Works thereof, You may choose to offer, and charge a fee for, acceptance of support, warranty, indemnity, or other liability obligations and/or rights consistent with this License. However, in accepting such obligations, You may act only on Your own behalf and on Your sole responsibility, not on behalf of any other Contributor, and only if You agree to indemnify, defend, and hold each Contributor harmless for any liability incurred by, or claims asserted against, such Contributor by reason of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

---

## Installation & Build Setup

Add `qpipeline` to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
qpipeline = "1.3.0"
```

Build the static and dynamic libraries (`.dll` on Windows, `.so` on Linux, `.dylib` on macOS):

```bash
cargo build --release
```

---

## Usage in Rust

### Basic Example

```rust
use qpipeline::builder::QuatPipeline;
use qpipeline::chunk::QuatChunk;
use qpipeline::stage::{QuatInvertStage, QuatXorMaskStage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create pipeline and push stages
    let mut pipeline = QuatPipeline::new();
    pipeline.push_stage(QuatInvertStage);
    pipeline.push_stage(QuatXorMaskStage { mask: 0x0F0F_0F0F_0F0F_0F0F });

    // 2. Prepare dataset
    let mut chunks = vec![QuatChunk::new(0); 1_000_000];

    // 3. Process dataset in parallel
    pipeline.execute_smart(&mut chunks)?;

    println!("Processed {} chunks successfully!", chunks.len());
    Ok(())
}
```

### Advanced Example (Lookup Table Stage)

```rust
use qpipeline::builder::QuatPipeline;
use qpipeline::chunk::{QuatChunk, QuatState};
use qpipeline::stage::QuatLookupStage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create pipeline with a custom Lookup Table stage
    let mut pipeline = QuatPipeline::new();

    let lookup_stage = QuatLookupStage::new([
        QuatState::Q3,
        QuatState::Q2,
        QuatState::Q1,
        QuatState::Q0,
    ]);
    pipeline.push_stage(lookup_stage);

    // 2. Prepare dataset
    let mut chunks = vec![QuatChunk::new(0); 1_000_000];

    // 3. Execute processing
    pipeline.execute_smart(&mut chunks)?;
    println!("Advanced processing completed successfully!");
    Ok(())
}
```

### Expert Example (SIMD & Extended Quaternary Logic)

```rust
use qpipeline::chunk::{QuatChunk, QuatState};
use qpipeline::logic::QuatLogicOps;
use qpipeline::simd::SimdEngine;

fn main() {
    let q1 = QuatState::Q1;
    let q2 = QuatState::Q2;

    // Using Łukasiewicz quaternary logic operators
    let min_val = q1.min(q2);
    let negated = q1.negate();
    println!("Min: {:?}, Negated: {:?}", min_val, negated);

    // Using SIMD / Portable batch vectorization engine
    let mut chunks = vec![QuatChunk::new(0x55555555); 1000];
    SimdEngine::invert_portable(&mut chunks);
    println!("SIMD batch processed {} chunks.", chunks.len());
}
```

### Multivalued Logic, Truth Tables & Matrix Algebra

```rust
use qpipeline::chunk::QuatState;
use qpipeline::logic::{QuatLogicOps, QuatMatrix, QuatTruthTable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = QuatState::Q2;
    let b = QuatState::Q1;

    let luk = a.imply_lukasiewicz(b);
    let godel = a.imply_godel(b);
    let cons = a.consensus(b);
    println!("Luk: {luk:?}, Godel: {godel:?}, Consensus: {cons:?}");

    let table = QuatTruthTable::from_fn(|x, y| x.imply_lukasiewicz(y));
    println!("Table[Q0][Q1] = {:?}", table.evaluate(QuatState::Q0, QuatState::Q1));

    let m1 = QuatMatrix::new(2, 2, QuatState::Q2);
    let m2 = QuatMatrix::new(2, 2, QuatState::Q1);
    let product = m1.mul_min_max(&m2)?;
    println!("Matrix[0][0] = {:?}", product.get(0, 0));

    Ok(())
}
```

### Generic Block Pipeline & Execution Telemetry

```rust
use qpipeline::builder::{GenericQuatPipeline, GenericStage, PipelineTelemetry, QuatPipeline};
use qpipeline::chunk::{QuatBlock, QuatChunk, QuatDoubleChunk};
use qpipeline::error::PipelineError;
use qpipeline::stage::{QuatInvertStage, QuatStage};

struct GenericInvert;
impl<B: QuatBlock> GenericStage<B> for GenericInvert {
    fn process(&self, block: B) -> Result<B, PipelineError> {
        Ok(block.invert())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut double_chunks = vec![QuatDoubleChunk::new(0); 1_000];
    let generic_pipe = GenericQuatPipeline::<QuatDoubleChunk>::new().add_stage(GenericInvert);
    generic_pipe.execute_slice(&mut double_chunks)?;

    let mut pipeline = QuatPipeline::new();
    pipeline.push_stage(QuatInvertStage);

    let mut chunks = vec![QuatChunk::new(0); 10_000];
    QuatInvertStage.process_batch(&mut chunks)?;

    let telemetry: PipelineTelemetry = pipeline.execute_smart_with_telemetry(&mut chunks, 2048)?;
    println!(
        "Chunks: {}, Threads: {}, Elapsed ns: {}",
        telemetry.processed_chunks, telemetry.threads_used, telemetry.elapsed_nanos
    );

    Ok(())
}
```

---

## Foreign Language Support (FFI)

The FFI module exports C-compatible functions using `#[unsafe(no_mangle)]` and `extern "C"`. All foreign function calls are protected against panics to guarantee ABI safety.

### C / C++ Integration

Header file (`qpipeline.h`):

```c
#ifndef QPIPELINE_H
#define QPIPELINE_H

#include <stdint.h>
#include <stddef.h>

typedef enum QuatFFIStatus {
    QUAT_OK = 0,
    QUAT_NULL_POINTER = 1,
    QUAT_EXECUTION_ERROR = 2,
    QUAT_PANIC_OCCURRED = 3
} QuatFFIStatus;

typedef void* QuatPipelineHandle;

#ifdef __cplusplus
extern "C" {
#endif

QuatPipelineHandle qpipeline_create(void);
void qpipeline_free(QuatPipelineHandle ptr);
QuatFFIStatus qpipeline_add_invert_stage(QuatPipelineHandle ptr);
QuatFFIStatus qpipeline_add_xor_stage(QuatPipelineHandle ptr, uint64_t mask);
QuatFFIStatus qpipeline_execute_smart(QuatPipelineHandle ptr, uint64_t* chunks_ptr, size_t len);

#ifdef __cplusplus
}
#endif

#endif // QPIPELINE_H
```

Example C usage:

```c
#include "qpipeline.h"
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    QuatPipelineHandle pipeline = qpipeline_create();
    if (!pipeline) {
        fprintf(stderr, "Failed to create pipeline\n");
        return 1;
    }

    if (qpipeline_add_invert_stage(pipeline) != QUAT_OK ||
        qpipeline_add_xor_stage(pipeline, 0xFF00FF00FF00FF00ULL) != QUAT_OK) {
        fprintf(stderr, "Failed to add pipeline stages\n");
        qpipeline_free(pipeline);
        return 1;
    }

    const size_t len = 10000;
    uint64_t *data = (uint64_t *)calloc(len, sizeof(uint64_t));
    if (!data) {
        qpipeline_free(pipeline);
        return 1;
    }

    QuatFFIStatus status = qpipeline_execute_smart(pipeline, data, len);
    if (status == QUAT_OK) {
        printf("Data processed successfully via C!\n");
    } else {
        fprintf(stderr, "Pipeline execution failed with status: %d\n", (int)status);
    }

    free(data);
    qpipeline_free(pipeline);
    return (status == QUAT_OK) ? 0 : 1;
}
```

Example C++ usage:

```cpp
#include "qpipeline.h"
#include <vector>
#include <iostream>

int main() {
    QuatPipelineHandle pipeline = qpipeline_create();

    qpipeline_add_invert_stage(pipeline);
    qpipeline_add_xor_stage(pipeline, 0xFF00FF00FF00FF00ULL);

    std::vector<uint64_t> data(10000, 0);
    QuatFFIStatus status = qpipeline_execute_smart(pipeline, data.data(), data.size());

    if (status == QUAT_OK) {
        std::cout << "Data processed successfully via C++!" << std::endl;
    }

    qpipeline_free(pipeline);
    return 0;
}
```

---

### C# Integration (P/Invoke)

```csharp
using System;
using System.Runtime.InteropServices;

public enum QuatFFIStatus
{
    Ok = 0,
    NullPointer = 1,
    ExecutionError = 2,
    PanicOccurred = 3
}

public static class NativeQPipeline
{
    private const string DllName = "qpipeline.dll"; // or libqpipeline.so / libqpipeline.dylib

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr qpipeline_create();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void qpipeline_free(IntPtr ptr);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern QuatFFIStatus qpipeline_add_invert_stage(IntPtr ptr);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern QuatFFIStatus qpipeline_add_xor_stage(IntPtr ptr, ulong mask);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern QuatFFIStatus qpipeline_execute_smart(IntPtr ptr, ulong[] chunks, UIntPtr len);
}

class Program
{
    static void Main()
    {
        IntPtr pipeline = NativeQPipeline.qpipeline_create();

        NativeQPipeline.qpipeline_add_invert_stage(pipeline);
        NativeQPipeline.qpipeline_add_xor_stage(pipeline, 0xAAAAAAAA);

        ulong[] data = new ulong[100000];
        QuatFFIStatus status = NativeQPipeline.qpipeline_execute_smart(pipeline, data, (UIntPtr)data.Length);

        if (status == QuatFFIStatus.Ok)
        {
            Console.WriteLine("Successfully executed Rust pipeline from C#!");
        }

        NativeQPipeline.qpipeline_free(pipeline);
    }
}
```

---

## Testing & Benchmarks

Run unit and stress tests:

```bash
cargo test
```

Run tests with console output enabled:

```bash
cargo test -- --nocapture
```

---

## Licensing

This project is dual-licensed under:

1. **GNU Affero General Public License v3.0 (AGPL-3.0)** for open-source use.
2. **Commercial License** for proprietary/commercial applications.

### Commercial License
If you wish to use `qpipeline` in closed-source software, proprietary products, or environments where AGPL-3.0 compliance is not feasible (e.g., embedding DLLs/shared libraries without disclosing host source code), a commercial license is required.

For commercial licensing inquiries, please contact: `strahinjastojanovic826@gmail.com`

---