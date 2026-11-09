//! The cxx bridge to the C++ shim in `src/cxx`.
//!
//! Declaring the C++ functions in an `unsafe extern "C++"` block asserts that
//! they are safe to call with any arguments Rust can pass. The shim upholds
//! that: it checks shared enum values, iterator validity and column family
//! ownership instead of relying on RocksDB's debug assertions. Together with
//! the `Send`/`Sync` impls below, this is the only `unsafe` in the crate.
//!
//! The safe API lives in the sibling modules, which only re-export the shared
//! enums and `ColumnFamily` from here.

use crate::slice_transform::RustSliceTransform;

#[cxx::bridge(namespace = "rocksdb_rs")]
mod bridge {
    #[derive(Debug)]
    #[repr(u8)]
    enum StatusCode {
        Ok,
        NotFound,
        Corruption,
        NotSupported,
        InvalidArgument,
        IOError,
        MergeInProgress,
        Incomplete,
        ShutdownInProgress,
        TimedOut,
        Aborted,
        Busy,
        Expired,
        TryAgain,
        CompactionTooLarge,
        ColumnFamilyDropped,
        Unknown,
    }

    /// The outcome of a RocksDB call, passed as an out parameter. Callers
    /// initialize it as OK and C++ only overwrites it on errors.
    struct Status {
        code: StatusCode,
        message: String,
    }

    /// A subslice of a key, which C++ can hold without a Rust lifetime.
    #[derive(Debug, PartialEq, Eq)]
    struct KeyRange {
        start: usize,
        len: usize,
    }

    #[repr(u8)]
    enum OpenMode {
        ReadWrite,
        ReadOnly,
        Secondary,
    }

    /// Compression algorithms. LZ4 is linked in with the `lz4` feature.
    #[derive(Debug)]
    #[repr(u8)]
    enum DBCompressionType {
        None,
        Lz4,
        Lz4hc,
    }

    #[derive(Debug)]
    #[repr(u8)]
    enum BlockBasedIndexType {
        /// A space efficient index block that is optimized for binary search.
        BinarySearch,
        /// A hash index on top of binary search. Requires a prefix extractor.
        HashSearch,
        /// A two-level index. Both levels are binary search indexes.
        TwoLevelIndexSearch,
    }

    /// Controls the overhead of collecting statistics.
    #[derive(Debug)]
    #[repr(u8)]
    enum StatsLevel {
        /// Disable all metrics.
        DisableAll,
        /// Disable timer stats, and skip histogram stats.
        ExceptHistogramOrTimers,
        /// Skip timer stats.
        ExceptTimers,
        /// Collect all stats except time inside mutex lock and time spent on
        /// compression.
        ExceptDetailedTimers,
        /// Collect all stats except the counters requiring to get time inside
        /// the mutex lock.
        ExceptTimeForMutex,
        /// Collect all stats, including measuring duration of mutex operations.
        All,
    }

    /// Cumulative counters collected when statistics are enabled. See
    /// `rocksdb/statistics.h` for their meaning.
    #[derive(Debug)]
    #[repr(u32)]
    enum Ticker {
        BlockCacheMiss,
        BlockCacheHit,
        BlockCacheIndexMiss,
        BlockCacheIndexHit,
        BlockCacheFilterMiss,
        BlockCacheFilterHit,
        BlockCacheDataMiss,
        BlockCacheDataHit,
        BloomFilterUseful,
        BloomFilterFullPositive,
        BloomFilterFullTruePositive,
        BloomFilterPrefixChecked,
        BloomFilterPrefixUseful,
        BloomFilterPrefixTruePositive,
        StallMicros,
        CompactReadBytes,
        CompactWriteBytes,
        FlushWriteBytes,
        LastLevelSeekFiltered,
        LastLevelSeekFilterMatch,
        LastLevelSeekData,
        LastLevelSeekDataUsefulNoFilter,
        LastLevelSeekDataUsefulFilterMatch,
        NonLastLevelSeekFiltered,
        NonLastLevelSeekFilterMatch,
        NonLastLevelSeekData,
        NonLastLevelSeekDataUsefulNoFilter,
        NonLastLevelSeekDataUsefulFilterMatch,
    }

    extern "Rust" {
        type RustSliceTransform;
        fn name(self: &RustSliceTransform) -> &str;
        fn transform(self: &RustSliceTransform, key: &[u8]) -> KeyRange;
        fn in_domain(self: &RustSliceTransform, key: &[u8]) -> bool;

        fn extend_from_slice(vec: &mut Vec<u8>, data: &[u8]);
    }

    #[namespace = "rocksdb"]
    unsafe extern "C++" {
        include!("rocksdb/cache.h");
        include!("rocksdb/env.h");

        type Cache;
        type Env;
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/cache.h");

        fn new_hyper_clock_cache(
            capacity: usize,
            estimated_entry_charge: usize,
        ) -> SharedPtr<Cache>;
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/env.h");

        fn default_env() -> SharedPtr<Env>;
        fn set_low_priority_background_threads(env: &SharedPtr<Env>, num: i32);
        fn set_high_priority_background_threads(env: &SharedPtr<Env>, num: i32);
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/options.h");

        type BlockBasedOptions;
        fn new_block_based_options() -> UniquePtr<BlockBasedOptions>;
        fn set_block_size(self: Pin<&mut BlockBasedOptions>, size: usize);
        fn set_index_type(self: Pin<&mut BlockBasedOptions>, index_type: BlockBasedIndexType);
        fn set_partition_filters(self: Pin<&mut BlockBasedOptions>, value: bool);
        fn set_cache_index_and_filter_blocks(self: Pin<&mut BlockBasedOptions>, value: bool);
        fn set_pin_l0_filter_and_index_blocks_in_cache(
            self: Pin<&mut BlockBasedOptions>,
            value: bool,
        );
        fn set_block_cache(self: Pin<&mut BlockBasedOptions>, cache: &SharedPtr<Cache>);
        fn set_bloom_filter(self: Pin<&mut BlockBasedOptions>, bits_per_key: f64);
        fn set_hybrid_ribbon_filter(
            self: Pin<&mut BlockBasedOptions>,
            bloom_equivalent_bits_per_key: f64,
            bloom_before_level: i32,
        );
        fn set_whole_key_filtering(self: Pin<&mut BlockBasedOptions>, value: bool);

        type Options;
        fn new_options() -> UniquePtr<Options>;
        fn clone_options(options: &Options) -> UniquePtr<Options>;
        fn set_create_if_missing(self: Pin<&mut Options>, value: bool);
        fn set_create_missing_column_families(self: Pin<&mut Options>, value: bool);
        fn set_max_open_files(self: Pin<&mut Options>, value: i32);
        fn set_max_total_wal_size(self: Pin<&mut Options>, value: u64);
        fn set_max_background_jobs(self: Pin<&mut Options>, value: i32);
        fn set_stats_dump_period_sec(self: Pin<&mut Options>, value: u32);
        fn set_env(self: Pin<&mut Options>, env: &SharedPtr<Env>);
        fn enable_statistics(self: Pin<&mut Options>);
        fn set_statistics_level(self: Pin<&mut Options>, level: StatsLevel);
        fn get_ticker_count(self: &Options, ticker: Ticker) -> u64;
        fn set_compression_type(self: Pin<&mut Options>, compression: DBCompressionType);
        fn set_block_based_table_factory(self: Pin<&mut Options>, options: &BlockBasedOptions);
        fn add_compact_on_deletion_collector_factory(
            self: Pin<&mut Options>,
            window_size: usize,
            num_dels_trigger: usize,
            deletion_ratio: f64,
        );
        fn set_soft_pending_compaction_bytes_limit(self: Pin<&mut Options>, value: u64);
        fn set_hard_pending_compaction_bytes_limit(self: Pin<&mut Options>, value: u64);
        fn set_level_zero_slowdown_writes_trigger(self: Pin<&mut Options>, value: i32);
        fn set_level_zero_stop_writes_trigger(self: Pin<&mut Options>, value: i32);
        fn set_prefix_extractor(self: Pin<&mut Options>, transform: Box<RustSliceTransform>);

        type ReadOptions;
        fn new_read_options() -> UniquePtr<ReadOptions>;
        fn fill_cache(self: Pin<&mut ReadOptions>, value: bool);
        fn set_prefix_same_as_start(self: Pin<&mut ReadOptions>, value: bool);
        fn set_total_order_seek(self: Pin<&mut ReadOptions>, value: bool);
        fn set_max_skippable_internal_keys(self: Pin<&mut ReadOptions>, value: u64);
        fn set_iterate_upper_bound(self: Pin<&mut ReadOptions>, key: &[u8]);

        type WriteOptions;
        fn new_write_options() -> UniquePtr<WriteOptions>;
        fn set_sync(self: Pin<&mut WriteOptions>, value: bool);
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/column_family.h");

        /// A handle to an open column family, borrowed from a `DB`.
        type ColumnFamily;

        type ColumnFamilyDescriptors;
        fn new_column_family_descriptors() -> UniquePtr<ColumnFamilyDescriptors>;
        fn add(self: Pin<&mut ColumnFamilyDescriptors>, name: &str, options: &Options);
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/write_batch.h");

        type WriteBatch;
        fn new_write_batch() -> UniquePtr<WriteBatch>;
        fn put(self: Pin<&mut WriteBatch>, key: &[u8], value: &[u8], status: &mut Status);
        fn put_cf(
            self: Pin<&mut WriteBatch>,
            cf: &ColumnFamily,
            key: &[u8],
            value: &[u8],
            status: &mut Status,
        );
        fn del(self: Pin<&mut WriteBatch>, key: &[u8], status: &mut Status);
        fn del_cf(self: Pin<&mut WriteBatch>, cf: &ColumnFamily, key: &[u8], status: &mut Status);
        fn clear(self: Pin<&mut WriteBatch>);
        fn count(self: &WriteBatch) -> u32;
        fn data_size(self: &WriteBatch) -> usize;
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/iterator.h");

        /// An iterator over a column family, borrowing the `Db` it reads.
        type DbIterator<'a>;
        fn valid(self: &DbIterator) -> bool;
        fn seek_to_first(self: Pin<&mut DbIterator>);
        fn seek_to_last(self: Pin<&mut DbIterator>);
        fn seek(self: Pin<&mut DbIterator>, key: &[u8]);
        fn seek_for_prev(self: Pin<&mut DbIterator>, key: &[u8]);
        fn next(self: Pin<&mut DbIterator>);
        fn prev(self: Pin<&mut DbIterator>);
        fn key(self: &DbIterator) -> &[u8];
        fn value(self: &DbIterator) -> &[u8];
        fn status(self: &DbIterator, status: &mut Status);
    }

    unsafe extern "C++" {
        include!("rocksdb-rs/src/cxx/db.h");

        type Db;
        fn open_db(
            options: &Options,
            path: &[u8],
            column_families: &ColumnFamilyDescriptors,
            mode: OpenMode,
            secondary_path: &[u8],
            error_if_wal_file_exists: bool,
            status: &mut Status,
        ) -> UniquePtr<Db>;
        fn list_column_families(options: &Options, path: &[u8], status: &mut Status)
        -> Vec<String>;
        fn column_family(self: &Db, index: usize) -> &ColumnFamily;
        fn get(
            self: &Db,
            options: &ReadOptions,
            cf: &ColumnFamily,
            key: &[u8],
            value: &mut Vec<u8>,
            status: &mut Status,
        ) -> bool;
        fn put(
            self: &Db,
            options: &WriteOptions,
            cf: &ColumnFamily,
            key: &[u8],
            value: &[u8],
            status: &mut Status,
        );
        fn del(
            self: &Db,
            options: &WriteOptions,
            cf: &ColumnFamily,
            key: &[u8],
            status: &mut Status,
        );
        fn write(
            self: &Db,
            options: &WriteOptions,
            batch: Pin<&mut WriteBatch>,
            status: &mut Status,
        );
        fn flush(self: &Db, cf: &ColumnFamily, status: &mut Status);
        fn get_int_property(
            self: &Db,
            cf: &ColumnFamily,
            name: &str,
            value: &mut u64,
            status: &mut Status,
        ) -> bool;
        fn new_iterator<'a>(
            self: &'a Db,
            options: UniquePtr<ReadOptions>,
            cf: &ColumnFamily,
        ) -> UniquePtr<DbIterator<'a>>;
        fn create_checkpoint(self: &Db, path: &[u8], status: &mut Status);
    }
}

pub(crate) use bridge::*;
pub use bridge::{BlockBasedIndexType, ColumnFamily, DBCompressionType, StatsLevel, Ticker};

fn extend_from_slice(vec: &mut Vec<u8>, data: &[u8]) {
    vec.extend_from_slice(data);
}

impl Status {
    pub(crate) fn ok() -> Self {
        Self {
            code: StatusCode::Ok,
            message: String::new(),
        }
    }
}

// SAFETY: RocksDB caches and envs are internally synchronized and meant to be
// shared across threads and DBs.
unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}
unsafe impl Send for Env {}
unsafe impl Sync for Env {}

// SAFETY: Options are plain data, only mutated through `Pin<&mut _>`. What
// they point to (caches, statistics, prefix extractors) is thread-safe.
unsafe impl Send for BlockBasedOptions {}
unsafe impl Sync for BlockBasedOptions {}
unsafe impl Send for Options {}
unsafe impl Sync for Options {}
unsafe impl Send for ReadOptions {}
unsafe impl Sync for ReadOptions {}
unsafe impl Send for WriteOptions {}
unsafe impl Sync for WriteOptions {}

// SAFETY: Plain data, only mutated through `Pin<&mut _>`.
unsafe impl Send for WriteBatch {}
unsafe impl Sync for WriteBatch {}

// SAFETY: `rocksdb::DB` and its column family handles are thread-safe.
unsafe impl Send for Db {}
unsafe impl Sync for Db {}
unsafe impl Sync for ColumnFamily {}

// SAFETY: RocksDB iterators can move between threads, but are not
// synchronized for concurrent use.
unsafe impl Send for DbIterator<'_> {}
