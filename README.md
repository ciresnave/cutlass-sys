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
- ✅ **Persistent Caching**: Downloads cached in `$CARGO_HOME` or system cache dir
- ✅ **Retry Logic**: Configurable retries with exponential backoff for reliability
- ✅ **Git Fallback**: Automatically tries `git clone` if HTTP download fails
- ✅ **Local Override**: Use `CUTLASS_DIR` to skip downloads entirely
- ✅ **Cargo Integration**: Works seamlessly with Cargo's dependency resolution
- ✅ **Cross-Platform**: Handles Windows, Linux, and macOS
- ✅ **Build Caching**: Downloads are cached between builds

## Versioning

This crate's version **matches the CUTLASS version** it provides:

- `cutlass-sys = "4.2.0"` → CUTLASS 4.2.0 (Latest, Hopper/Blackwell support)
- `cutlass-sys = "3.9.2"` → CUTLASS 3.9.2 (Mature 3.x)
- `cutlass-sys = "3.5.0"` → CUTLASS 3.5.0 (Candle-compatible)
- `cutlass-sys = "2.11.0"` → CUTLASS 2.11.0 (Legacy 2.x, Volta/Ampere)

### Release Candidates

New versions are first released as **release candidates** (e.g., `4.2.0-rc.1`) for testing. These allow us to verify the crate works correctly before committing to the stable version number.

- RC versions must be explicitly requested: `cutlass-sys = "4.2.0-rc.1"`
- Cargo will not auto-select RCs when you use `cutlass-sys = "4.2"`
- Once tested, we publish the stable version (e.g., `4.2.0`)

**Need a different CUTLASS version?** Open an issue and we can publish it!

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
    // Try multiple possible environment variable names for maximum compatibility
    let cutlass_include = std::env::var("DEP_CUTLASS_SYS_INCLUDE")
        .or_else(|_| std::env::var("DEP_CUTLASS_SYS_INCLUDE_DIR"))
        .or_else(|_| std::env::var("CUTLASS_INCLUDE_DIR"))
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

## Environment Variables

### Build-Time Configuration

- **`CUTLASS_DIR`**: Point to a local CUTLASS installation to skip downloads entirely
  ```bash
  CUTLASS_DIR=/path/to/cutlass cargo build
  ```

- **`CUTLASS_DOWNLOAD_TIMEOUT`**: Download timeout in seconds (default: 120)
  ```bash
  CUTLASS_DOWNLOAD_TIMEOUT=300 cargo build
  ```

- **`CUTLASS_DOWNLOAD_RETRIES`**: Number of retry attempts (default: 3)
  ```bash
  CUTLASS_DOWNLOAD_RETRIES=5 cargo build
  ```

### For Dependent Crates (Build Scripts)

When you depend on `cutlass-sys`, the following environment variables are available in your `build.rs`:

- `DEP_CUTLASS_SYS_ROOT`: Root directory of CUTLASS installation
- `DEP_CUTLASS_SYS_INCLUDE`: Include directory path (same as `INCLUDE_DIR`)
- `DEP_CUTLASS_SYS_INCLUDE_DIR`: Include directory path (recommended)
- `CUTLASS_INCLUDE_DIR`: Also available via `cargo:rustc-env`
- `CUTLASS_ROOT`: Root directory via `cargo:rustc-env`

**Recommended consumer code for maximum compatibility:**

```rust
// In your build.rs
let cutlass_include = std::env::var("DEP_CUTLASS_SYS_INCLUDE_DIR")
    .or_else(|_| std::env::var("DEP_CUTLASS_SYS_INCLUDE"))
    .or_else(|_| std::env::var("CUTLASS_INCLUDE_DIR"))
    .expect("cutlass-sys not found");

// Keep the formatted string alive until after builder.build()
let include_arg = format!("-I{}", cutlass_include);
builder = builder.arg(&include_arg);
// ... other config
builder.build().unwrap();
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
3. The build script checks for CUTLASS in this order:
   - `CUTLASS_DIR` environment variable (if set)
   - Persistent cache (`$CARGO_HOME/cutlass-sys-cache` or `~/.cache/cutlass-sys`)
   - Download from GitHub (with retry and git fallback)
4. The include path is exported via multiple `cargo:` keys for maximum compatibility
5. Your `build.rs` can access these paths via `DEP_CUTLASS_SYS_*` environment variables
6. You can then use CUTLASS in your CUDA/C++ code

## Troubleshooting

### Download Timeouts or Network Errors

If builds fail with timeout errors:

1. **Increase timeout**:
   ```bash
   CUTLASS_DOWNLOAD_TIMEOUT=300 cargo build
   ```

2. **Use a local copy**:
   ```bash
   git clone --depth 1 --branch v4.2.1 https://github.com/NVIDIA/cutlass.git
   CUTLASS_DIR=./cutlass cargo build
   ```

3. **Check cache**: CUTLASS is cached after first download. Clear cache if corrupted:
   ```bash
   rm -rf ~/.cargo/cutlass-sys-cache  # or ~/.cache/cutlass-sys
   ```

### Missing Headers in Consumer Crates

If your crate can't find CUTLASS headers:

1. **Check environment variables** in your `build.rs`:
   ```rust
   // Add this debug code temporarily
   for (k, v) in std::env::vars().filter(|(k, _)| k.contains("CUTLASS")) {
       println!("cargo:warning={}={}", k, v);
   }
   ```

2. **Use multiple fallbacks**:
   ```rust
   let include = std::env::var("DEP_CUTLASS_SYS_INCLUDE_DIR")
       .or_else(|_| std::env::var("DEP_CUTLASS_SYS_INCLUDE"))
       .or_else(|_| std::env::var("CUTLASS_INCLUDE_DIR"))
       .expect("cutlass-sys not found");
   ```

3. **Keep strings alive**: When using `format!()`, store in a variable:
   ```rust
   let include_arg = format!("-I{}", cutlass_include);
   builder = builder.arg(&include_arg);  // Not: .arg(format!(...))
   ```

### CI/Offline Builds

For reproducible CI builds without network access:

```yaml
# .github/workflows/build.yml
- name: Cache CUTLASS
  uses: actions/cache@v3
  with:
    path: ~/.cargo/cutlass-sys-cache
    key: cutlass-${{ matrix.cutlass-version }}

- name: Build
  run: cargo build
```

Or use a pre-cloned CUTLASS:

```yaml
- name: Clone CUTLASS
  run: git clone --depth 1 --branch v4.2.1 https://github.com/NVIDIA/cutlass.git

- name: Build
  env:
    CUTLASS_DIR: ./cutlass
  run: cargo build
```

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
