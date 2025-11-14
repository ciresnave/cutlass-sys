//! # cutlass-sys
//!
//! Raw FFI bindings for NVIDIA CUTLASS (CUDA Templates for Linear Algebra Subroutines).
//!
//! This crate provides a Rust-friendly way to depend on CUTLASS headers without
//! manually managing git submodules. The build script automatically downloads
//! the CUTLASS headers from GitHub.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cutlass-sys = "0.1"
//! ```
//!
//! In your `build.rs`, you can access the CUTLASS include directory:
//!
//! ```rust,no_run
//! fn main() {
//!     let cutlass_include = std::env::var("DEP_CUTLASS_INCLUDE_DIR")
//!         .expect("cutlass-sys should set this");
//!     
//!     println!("cargo:rustc-link-search=native={}", cutlass_include);
//!     // Use cutlass_include in your cc::Build configuration
//! }
//! ```
//!
//! ## Environment Variables
//!
//! - `CUTLASS_VERSION`: Override the CUTLASS version to download (e.g., `v3.5.1`)
//!
//! ## Note
//!
//! This is a header-only library wrapper. No actual Rust bindings are provided,
//! but the headers are made available for use in your own build scripts with
//! `cc` or `bindgen`.

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// This crate is intentionally minimal - it exists primarily to manage
// CUTLASS headers as a Cargo dependency. The actual C++/CUDA code is
// header-only and will be included by dependent crates via their build scripts.
