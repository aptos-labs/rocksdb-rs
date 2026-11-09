//! Library sources, read from RocksDB's `src.mk` so that upgrading RocksDB
//! does not require maintaining a file list here.

use std::{fs, path::Path};

/// Make variables in `src.mk` that make up `librocksdb.a`, as in RocksDB's
/// Makefile. `LIB_SOURCES_ASM` and `LIB_SOURCES_C` only apply to ppc64, which
/// is not supported.
const LIB_SOURCE_VARS: &[&str] = &["LIB_SOURCES", "RANGE_TREE_SOURCES"];

/// Replaced by the generated file from [`crate::version`].
const EXCLUDED: &[&str] = &["util/build_version.cc"];

pub fn lib_sources(rocksdb_dir: &Path) -> Vec<String> {
    let src_mk = fs::read_to_string(rocksdb_dir.join("src.mk")).expect("failed to read src.mk");
    LIB_SOURCE_VARS
        .iter()
        .flat_map(|var| {
            make_variable(&src_mk, var).unwrap_or_else(|| panic!("`{var}` not found in src.mk"))
        })
        .filter(|source| !EXCLUDED.contains(&source.as_str()))
        .collect()
}

/// Returns the words assigned to `name` by a `NAME = a b \` assignment that
/// may continue over multiple lines. This is the only form `src.mk` uses for
/// the variables we read.
fn make_variable(makefile: &str, name: &str) -> Option<Vec<String>> {
    let mut lines = makefile.lines();
    let mut line = lines.by_ref().find_map(|line| {
        let (lhs, rhs) = line.split_once('=')?;
        (lhs.trim() == name).then_some(rhs)
    })?;

    let mut words = Vec::new();
    loop {
        let (content, continued) = match line.trim_end().strip_suffix('\\') {
            Some(content) => (content, true),
            None => (line, false),
        };
        words.extend(content.split_whitespace().map(str::to_owned));
        if !continued {
            return Some(words);
        }
        line = lines.next()?;
    }
}
