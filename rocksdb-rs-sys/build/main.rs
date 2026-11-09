//! Compiles RocksDB from the `rocksdb` submodule into a static library.
//!
//! The build mirrors what RocksDB's own CMake build does on each platform, so
//! users don't need to inject extra `CXXFLAGS`:
//!
//! - [`sources`] takes the library sources from RocksDB's `src.mk`.
//! - [`target`] sets the OS defines and the CPU features of the Rust target.
//! - [`checks`] runs the compile-time feature checks from RocksDB's build.
//! - [`version`] generates `build_version.cc`.

mod checks;
mod sources;
mod target;
mod version;

use std::{env, path::PathBuf};
use target::Target;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let rocksdb_dir = manifest_dir.join("rocksdb");
    assert!(
        rocksdb_dir.join("src.mk").exists(),
        "RocksDB sources are missing from {}. Run `git submodule update --init`.",
        rocksdb_dir.display(),
    );

    println!("cargo:rerun-if-changed=build");
    println!("cargo:rerun-if-changed=rocksdb");

    let target = Target::from_env();
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .include(rocksdb_dir.join("include"))
        .include(&rocksdb_dir)
        // Assertions and debug-only checks in RocksDB are expensive, so they
        // stay off regardless of the Cargo profile.
        .define("NDEBUG", None)
        // CMake's default. Loading RocksDB extensions from shared libraries
        // is not supported.
        .define("ROCKSDB_NO_DYNAMIC_EXTENSION", None)
        // Third-party code; its warnings are not actionable here.
        .warnings(false)
        .cargo_warnings(false);

    configure_compression(&mut build);
    target.configure(&mut build);
    checks::configure(&mut build, &target, &out_dir);

    for source in sources::lib_sources(&rocksdb_dir) {
        build.file(rocksdb_dir.join(source));
    }
    build.file(version::generate(&rocksdb_dir, &out_dir));
    build.compile("rocksdb");

    println!("cargo:include={}", rocksdb_dir.join("include").display());
}

fn configure_compression(build: &mut cc::Build) {
    if env::var_os("CARGO_FEATURE_LZ4").is_some() {
        build.define("LZ4", None);
        build.include(env::var_os("DEP_LZ4_INCLUDE").expect("lz4-sys exports DEP_LZ4_INCLUDE"));
    }
}
