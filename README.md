# cutlass-sys

[![Crates.io](https://img.shields.io/crates/v/cutlass-sys.svg)](https://crates.io/crates/cutlass-sys)
[![Documentation](https://docs.rs/cutlass-sys/badge.svg)](https://docs.rs/cutlass-sys)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Rust FFI bindings for [NVIDIA CUTLASS](https://github.com/NVIDIA/cutlass) - CUDA Templates for Linear Algebra Subroutines.

## Overview

This crate provides a Cargo-native way to depend on CUTLASS headers without manually managing git submodules. The build script automatically downloads the CUTLASS library from GitHub, making it available for your CUDA/C++ code.

**CUTLASS** is a collection of CUDA C++ template abstractions for implementing high-performance matrix-matrix multiplication (GEMM) and related computations at all levels and scales within CUDA.

## Features

- ✅ **No Git Submodules**: Automatic download and caching of CUTLASS headers
- ✅ **Version Locked**: Each crate version corresponds to a specific CUTLASS version
- ✅ **Cargo Integration**: Works seamlessly with Cargo's dependency resolution
- ✅ **Cross-Platform**: Handles Windows, Linux, and macOS
- ✅ **Build Caching**: Downloads are cached between builds

## Versioning

This crate's version directly matches the CUTLASS version it provides:

- `cutlass-sys = "4.2"` → CUTLASS 4.2.1 (Latest stable, Hopper/Blackwell support)
- `cutlass-sys = "3.9"` → CUTLASS 3.9.2 (Mature 3.x)
- `cutlass-sys = "2.11"` → CUTLASS 2.11.0 (Legacy 2.x, Volta/Ampere support)

**Need a different version?** Open an issue and we can publish it!

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cutlass-sys = "4.2"

[build-dependencies]
cc = "1.0"
```

In your `build.rs`, access the CUTLASS include directory:

```rust
fn main() {
    // Get the CUTLASS include directory from cutlass-sys
    let cutlass_include = std::env::var("DEP_CUTLASS_INCLUDE_DIR")
        .expect("cutlass-sys should provide CUTLASS include path");
    
    // Use it in your CUDA/C++ builds
    cc::Build::new()
        .cuda(true)
        .flag("-std=c++17")
        .include(&cutlass_include)
        .file("src/kernels/my_kernel.cu")
        .compile("my_cuda_kernels");
    
    println!("cargo:rerun-if-changed=src/kernels/my_kernel.cu");
}
```

### Example CUDA Code

```cuda
// src/kernels/my_kernel.cu
#include <cutlass/cutlass.h>
#include <cutlass/gemm/device/gemm.h>

// Your CUDA kernel using CUTLASS...
```

## How It Works

1. When you add `cutlass-sys` as a dependency, its `build.rs` runs during your build
2. The crate version (e.g., `4.2.1`) automatically determines which CUTLASS version to download
3. The build script downloads CUTLASS headers from GitHub (if not already cached)
4. The include path is exported via Cargo's metadata system
5. Your `build.rs` can access this path via `DEP_CUTLASS_INCLUDE_DIR`
6. You can then use CUTLASS in your CUDA/C++ code

## Example Project Structure

```
my-cuda-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.rs
    └── kernels/
        └── gemm.cu
```

**Cargo.toml**:
```toml
[dependencies]
cutlass-sys = "4.2"
```

[build-dependencies]
cc = "1.0"
```

**build.rs**:
```rust
fn main() {
    let cutlass = std::env::var("DEP_CUTLASS_INCLUDE_DIR").unwrap();
    
    cc::Build::new()
        .cuda(true)
        .flag("-std=c++17")
        .include(cutlass)
        .file("src/kernels/gemm.cu")
        .compile("gemm");
}
```

## Requirements

- CUDA Toolkit (for compiling CUDA code that uses CUTLASS)
- C++17 compatible compiler
- Rust 2021 edition or later

## License

This crate is licensed under MIT OR Apache-2.0.

CUTLASS itself is licensed under the [3-clause BSD License](https://github.com/NVIDIA/cutlass/blob/main/LICENSE.txt).

## Related Projects

- [cutlass](https://github.com/NVIDIA/cutlass) - The official NVIDIA CUTLASS repository
- [bindgen](https://github.com/rust-lang/rust-bindgen) - For generating Rust bindings from C++ headers
- [cc](https://github.com/rust-lang/cc-rs) - For compiling C/C++/CUDA code in build scripts

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This crate wraps [NVIDIA CUTLASS](https://github.com/NVIDIA/cutlass), developed by NVIDIA Corporation.
