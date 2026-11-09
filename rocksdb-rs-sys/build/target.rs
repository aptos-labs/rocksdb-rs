//! Platform defines and CPU feature flags for the Cargo target.

use std::env;

/// Rust target features that RocksDB can take advantage of, and the GCC/Clang
/// flags enabling them. These keep the C++ code on the same instruction set
/// baseline as the Rust code, e.g. with `-C target-cpu=x86-64-v3`.
const X86_FEATURE_FLAGS: &[(&str, &str)] = &[
    ("sse4.2", "-msse4.2"),
    ("pclmulqdq", "-mpclmul"),
    ("popcnt", "-mpopcnt"),
    ("avx", "-mavx"),
    ("avx2", "-mavx2"),
    ("bmi1", "-mbmi"),
    ("bmi2", "-mbmi2"),
    ("lzcnt", "-mlzcnt"),
];

pub struct Target {
    pub os: String,
    pub arch: String,
    env: String,
    unix: bool,
    features: Vec<String>,
}

impl Target {
    pub fn from_env() -> Self {
        let var = |name| env::var(name).unwrap_or_default();
        Self {
            os: var("CARGO_CFG_TARGET_OS"),
            arch: var("CARGO_CFG_TARGET_ARCH"),
            env: var("CARGO_CFG_TARGET_ENV"),
            unix: var("CARGO_CFG_TARGET_FAMILY")
                .split(',')
                .any(|family| family == "unix"),
            features: var("CARGO_CFG_TARGET_FEATURE")
                .split(',')
                .map(str::to_owned)
                .collect(),
        }
    }

    pub fn is_x86(&self) -> bool {
        matches!(self.arch.as_str(), "x86" | "x86_64")
    }

    fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn configure(&self, build: &mut cc::Build) {
        self.configure_os(build);
        self.configure_cpu_features(build);

        let compiler = build.get_compiler();
        if !compiler.is_like_msvc() {
            // Same as RocksDB's CMake build in release mode. Keeps profilers
            // that unwind with frame pointers working.
            build.flag("-fno-omit-frame-pointer");
        }
        if compiler.is_like_gnu() {
            // Also from CMake: GCC's builtin is slower than libc's memcmp.
            build.flag("-fno-builtin-memcmp");
        }
    }

    /// Mirrors the OS defines in RocksDB's `CMakeLists.txt`.
    fn configure_os(&self, build: &mut cc::Build) {
        let os_define = match self.os.as_str() {
            "linux" => "OS_LINUX",
            "android" => "OS_ANDROID",
            "macos" | "ios" => "OS_MACOSX",
            "freebsd" => "OS_FREEBSD",
            "netbsd" => "OS_NETBSD",
            "openbsd" => "OS_OPENBSD",
            "dragonfly" => "OS_DRAGONFLYBSD",
            "solaris" | "illumos" => "OS_SOLARIS",
            "windows" => "OS_WIN",
            os => panic!("unsupported target OS: {os}"),
        };
        build.define(os_define, None);

        if self.os == "ios" {
            build.define("IOS_CROSS_COMPILE", None);
        }

        if self.unix {
            build
                .define("ROCKSDB_PLATFORM_POSIX", None)
                .define("ROCKSDB_LIB_IO_POSIX", None)
                .flag("-pthread");
        }

        if self.os == "windows" {
            build
                .define("WIN32", None)
                .define("WIN64", None)
                .define("_MBCS", None)
                .define("NOMINMAX", None)
                // Rust passes paths to RocksDB as UTF-8.
                .define("ROCKSDB_WINDOWS_UTF8_FILENAMES", None);
            if self.env == "gnu" {
                build
                    .define("_POSIX_C_SOURCE", Some("1"))
                    .define("_WIN32_WINNT", Some("_WIN32_WINNT_VISTA"));
            }
            for lib in ["rpcrt4", "shlwapi"] {
                println!("cargo:rustc-link-lib=dylib={lib}");
            }
        }
    }

    fn configure_cpu_features(&self, build: &mut cc::Build) {
        if !self.is_x86() {
            return;
        }
        if build.get_compiler().is_like_msvc() {
            if self.has_feature("avx2") {
                build.flag("/arch:AVX2");
            } else if self.has_feature("avx") {
                build.flag("/arch:AVX");
            }
        } else {
            for (feature, flag) in X86_FEATURE_FLAGS {
                if self.has_feature(feature) {
                    build.flag(flag);
                }
            }
        }
    }
}
