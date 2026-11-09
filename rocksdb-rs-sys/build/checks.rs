//! Compile-time checks against the configured compiler and flags.
//!
//! Most of these mirror `CHECK_CXX_SOURCE_COMPILES` and
//! `check_cxx_symbol_exists` in RocksDB's `CMakeLists.txt`. Without them,
//! RocksDB silently falls back to slower or less accurate code paths, e.g.
//! memory accounting without `ROCKSDB_MALLOC_USABLE_SIZE`.

use crate::target::Target;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

struct Check {
    define: &'static str,
    source: String,
}

impl Check {
    fn compiles(define: &'static str, source: &str) -> Self {
        Self {
            define,
            source: source.to_owned(),
        }
    }

    /// Same as CMake's `check_cxx_symbol_exists`, without linking.
    fn symbol(define: &'static str, header: &str, symbol: &str) -> Self {
        let source = format!(
            r#"#include <{header}>
int main(int argc, char** argv) {{
  (void)argv;
#ifndef {symbol}
  return ((int*)(&{symbol}))[argc];
#else
  (void)argc;
  return 0;
#endif
}}
"#
        );
        Self { define, source }
    }
}

fn checks(target: &Target) -> Vec<Check> {
    let malloc_header = if target.os == "freebsd" {
        "malloc_np.h"
    } else {
        "malloc.h"
    };
    // `ROCKSDB_FALLOCATE_PRESENT` is left out, like in `librocksdb-sys`.
    // Preallocating files can fill up small disks and tmpfs in tests, for
    // little benefit.
    vec![
        Check::compiles(
            "ROCKSDB_RANGESYNC_PRESENT",
            r#"#include <fcntl.h>
int main() {
  int fd = open("/dev/null", 0);
  sync_file_range(fd, 0, 1024, SYNC_FILE_RANGE_WRITE);
}
"#,
        ),
        Check::compiles(
            "ROCKSDB_PTHREAD_ADAPTIVE_MUTEX",
            r#"#include <pthread.h>
int main() {
  (void) PTHREAD_MUTEX_ADAPTIVE_NP;
}
"#,
        ),
        Check::symbol(
            "ROCKSDB_MALLOC_USABLE_SIZE",
            malloc_header,
            "malloc_usable_size",
        ),
        Check::symbol("ROCKSDB_SCHED_GETCPU_PRESENT", "sched.h", "sched_getcpu"),
        Check::symbol("ROCKSDB_AUXV_GETAUXVAL_PRESENT", "sys/auxv.h", "getauxval"),
        Check::symbol("HAVE_FULLFSYNC", "fcntl.h", "F_FULLFSYNC"),
        // Only RocksDB's Makefile build checks this. Without it, ribbon
        // filters and HyperClockCache emulate 128-bit arithmetic.
        Check::compiles(
            "HAVE_UINT128_EXTENSION",
            r#"#include <cstdint>
int main() {
  uint64_t a = 0xffffFFFFffffFFFF;
  __uint128_t b = __uint128_t(a) * a;
  a = static_cast<uint64_t>(b >> 64);
  (void)a;
}
"#,
        ),
    ]
}

pub fn configure(build: &mut cc::Build, target: &Target, out_dir: &Path) {
    let dir = out_dir.join("checks");
    fs::create_dir_all(&dir).expect("failed to create the checks directory");

    configure_cpu_features(build, target, &dir);

    let checks = checks(target);
    let results: Vec<_> = thread::scope(|s| {
        let handles: Vec<_> = checks
            .iter()
            .map(|check| {
                let build = build.clone();
                let dir = &dir;
                s.spawn(move || compiles(build, dir, check))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (check, passed) in checks.iter().zip(results) {
        if passed {
            build.define(check.define, None);
        }
    }
}

fn configure_cpu_features(build: &mut cc::Build, target: &Target, dir: &Path) {
    if build.get_compiler().is_like_msvc() {
        return;
    }
    if target.is_x86() {
        // RocksDB's `port/lang.h` assumes PCLMUL whenever AVX is available,
        // which holds for all AVX-capable CPUs. The compiler needs to agree,
        // e.g. with `-march=x86-64-v3`, which implies AVX but not PCLMUL.
        if defines_macro(build, dir, "__AVX__") && !defines_macro(build, dir, "__PCLMUL__") {
            build.flag("-mpclmul");
        }
    } else if target.arch == "aarch64"
        && !defines_macro(build, dir, "__ARM_FEATURE_CRC32")
        && build
            .is_flag_supported("-march=armv8-a+crc+crypto")
            .unwrap_or(false)
    {
        // Same as RocksDB's CMake build. CRC32C still checks for hardware
        // support at runtime.
        build.flag("-march=armv8-a+crc+crypto");
    }
}

fn compiles(mut build: cc::Build, dir: &Path, check: &Check) -> bool {
    let file = write_source(dir, check.define, &check.source);
    build
        .file(file)
        .out_dir(dir)
        .cargo_metadata(false)
        .try_compile_intermediates()
        .is_ok()
}

fn defines_macro(build: &cc::Build, dir: &Path, name: &str) -> bool {
    const MARKER: &str = "rocksdb_rs_macro_is_defined";
    let file = write_source(
        dir,
        &format!("macro{name}"),
        &format!("#ifdef {name}\n{MARKER}\n#endif\n"),
    );
    let output = build
        .clone()
        .file(file)
        .try_expand()
        .expect("failed to run the C++ preprocessor");
    String::from_utf8_lossy(&output).contains(MARKER)
}

fn write_source(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(format!("{name}.cc"));
    fs::write(&path, source).expect("failed to write check source");
    path
}
