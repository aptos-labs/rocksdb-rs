# rocksdb-rs

Rust bindings for [RocksDB](https://rocksdb.org), built on [cxx](https://cxx.rs).

> **Status: work in progress.** The API covers a deliberately small subset of
> RocksDB: opening DBs with column families (read-write, read-only and
> secondary), point reads and writes, write batches, iterators, block-based
> table options, block caches, prefix extractors, statistics, properties and
> checkpoints. It grows as needed.

```rust
use rocksdb_rs::{ColumnFamilyDescriptor, DB, Options, WriteBatch};

let mut opts = Options::default();
opts.create_if_missing(true);
opts.create_missing_column_families(true);
let db = DB::open_cf_descriptors(&opts, "/tmp/db", [
    ColumnFamilyDescriptor::new("users", Options::default()),
])?;

let users = db.cf_handle("users").unwrap();
let mut batch = WriteBatch::default();
batch.put_cf(users, b"alice", b"1")?;
batch.put_cf(users, b"bob", b"2")?;
db.write(batch)?;
assert_eq!(db.get_cf(users, b"alice")?, Some(b"1".to_vec()));
```

## Why cxx

Most Rust bindings for RocksDB go through its C API with `bindgen`. This crate
binds the C++ API directly through a thin C++ shim instead:

- **Safety.** cxx checks at compile time that both sides of the bridge agree on
  every signature, and expresses ownership and borrowing in the types
  (`UniquePtr`, `SharedPtr`, `Box`, references with lifetimes). The only
  `unsafe` in the crate is the bridge declaration and the `Send`/`Sync` impls
  in [`src/ffi.rs`](src/ffi.rs), each with a justification. The shim validates
  what RocksDB would otherwise only `assert` in debug builds, e.g. using an
  invalid iterator or a column family handle from another DB.
- **Coverage.** Anything in the C++ API can be exposed without waiting for a C
  API equivalent, such as Rust closures as prefix extractors.
- **Fewer copies.** For example, reads go through a `PinnableSlice`, so values
  in the block cache are copied only once, into the returned `Vec`.

## Layout

| Path | Contents |
| --- | --- |
| [`src/ffi.rs`](src/ffi.rs) | The cxx bridge: all C++ functions and shared types. |
| [`src/cxx/`](src/cxx) | The C++ shim, one file per component. |
| [`src/*.rs`](src) | The safe API, one module per component. |
| [`rocksdb-rs-sys/`](rocksdb-rs-sys) | Builds RocksDB from the [`rocksdb`](rocksdb-rs-sys/rocksdb) submodule and links it statically. |

Splitting out `rocksdb-rs-sys` means that changes to the bindings don't
rebuild RocksDB.

## Building

Requirements: a C++17 compiler and the RocksDB submodule.

```sh
git clone --recursive <repo>
# or, in an existing clone
git submodule update --init
cargo test
```

The `lz4` feature (on by default) links LZ4 for `DBCompressionType::Lz4` and
`Lz4hc`.

The build script aims to produce the same library as RocksDB's CMake build,
without extra `CFLAGS`/`CXXFLAGS`:

- Sources come from RocksDB's `src.mk`, so upgrades don't need a file list.
- It runs the platform checks from RocksDB's `CMakeLists.txt` against the
  configured compiler, enabling e.g. `ROCKSDB_MALLOC_USABLE_SIZE` for accurate
  memory accounting, and `sync_file_range` on Linux. It also checks for
  `__uint128_t` like the Makefile build, which CMake doesn't. `fallocate` is
  not used, since preallocated files can fill up small disks and tmpfs.
- The C++ code targets the same CPU features as the Rust code, e.g. with
  `-C target-cpu=x86-64-v3`. RocksDB assumes PCLMUL whenever AVX is enabled,
  so the build adds `-mpclmul` when the compiler flags don't.
- RocksDB is always built with `NDEBUG` and frame pointers.

RocksDB is slow without optimizations, which makes debug builds and tests
slow. Consider optimizing it in the dev profile:

```toml
[profile.dev.package.rocksdb-rs-sys]
opt-level = 2
```

## Extending

To expose more of RocksDB:

1. Declare the C++ functions in [`src/ffi.rs`](src/ffi.rs). Keep signatures
   safe for any arguments Rust can pass; the bridge asserts that they are.
2. Implement them in the matching files under [`src/cxx/`](src/cxx). cxx
   shared enums can hold any value, so map them to RocksDB's enums with a
   `switch` that handles unknown values.
3. Wrap them in the safe API and add tests next to the code.

## Upgrading RocksDB

1. Check out the new release tag in `rocksdb-rs-sys/rocksdb`.
2. Update the version in `rocksdb-rs-sys/Cargo.toml`.
3. Compare `CMakeLists.txt` and `build_tools/build_detect_platform` with
   [`rocksdb-rs-sys/build`](rocksdb-rs-sys/build) for new defines or checks,
   and run the tests.
