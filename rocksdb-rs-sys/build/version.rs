//! Generates `util/build_version.cc` from RocksDB's template, which CMake and
//! the Makefile would otherwise do.

use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn generate(rocksdb_dir: &Path, out_dir: &Path) -> PathBuf {
    let template = fs::read_to_string(rocksdb_dir.join("util/build_version.cc.in"))
        .expect("failed to read build_version.cc.in");
    // Git metadata placeholders are left as is. RocksDB skips properties
    // whose substitution failed.
    let contents = template
        .replace("@GIT_MOD@", "0")
        .replace("@ROCKSDB_PLUGIN_EXTERNS@", "")
        .replace("@ROCKSDB_PLUGIN_BUILTINS@", "");

    let path = out_dir.join("build_version.cc");
    fs::write(&path, contents).expect("failed to write build_version.cc");
    path
}
