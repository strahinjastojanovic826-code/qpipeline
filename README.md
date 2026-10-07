# QuatPipeline

`qpipeline` is a high-performance, thread-safe Rust library designed for fast, multi-threaded batch processing of quaternary data representations (`QuatChunk`). It includes zero-copy parallel processing pipelines, branchless bitwise transformations, and production-ready FFI (Foreign Function Interface) bindings for seamless integration with C, C++, and C#.

---

## Features

- **High-Performance Quaternary Data Operations**: Efficient bitwise manipulation of packed quaternary states.
- **Flexible Pipeline System**: Chain multiple execution stages (`QuatInvertStage`, `QuatXorMaskStage`, etc.) sequentially or staticaly.
- **Smart Parallel Processing**: Automatically scales workload across multiple CPU threads using `std::thread::scope` without runtime allocation overhead.
- **FFI Bindings for C, C++, and C#**: Safe, C-compatible binary interface with full panic boundaries (`catch_unwind`) to prevent crashes across foreign ABI boundaries.
- **Zero-Copy Architecture**: Process memory in-place directly over raw pointers or native slices.

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

## Installation & Build Setup

Add `qpipeline` to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Add crate dependencies here
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

For commercial licensing inquiries, please contact: `your.email@domain.com`

---