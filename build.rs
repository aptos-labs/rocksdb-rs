//! Compiles the cxx bridge and the C++ shim in `src/cxx`.

use std::{env, path::Path};

const BRIDGE: &str = "src/ffi.rs";
const SHIM_DIR: &str = "src/cxx";

fn main() {
    let rocksdb_include =
        env::var_os("DEP_ROCKSDB_INCLUDE").expect("rocksdb-rs-sys exports DEP_ROCKSDB_INCLUDE");

    let shim_sources: Vec<_> = Path::new(SHIM_DIR)
        .read_dir()
        .expect("failed to read the shim directory")
        .map(|entry| entry.expect("failed to read the shim directory").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cc"))
        .collect();

    let mut build = cxx_build::bridge(BRIDGE);
    build
        .files(shim_sources)
        .include(rocksdb_include)
        // Must match how RocksDB is built.
        .std("c++17");
    if !build.get_compiler().is_like_msvc() {
        build.flag("-Wno-unused-parameter");
    }
    build.compile("rocksdb-rs-shim");

    println!("cargo:rerun-if-changed={BRIDGE}");
    println!("cargo:rerun-if-changed={SHIM_DIR}");
}
