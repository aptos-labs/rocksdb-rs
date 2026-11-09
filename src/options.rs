mod block_based;
mod read;
mod write;

pub use self::{block_based::BlockBasedOptions, read::ReadOptions, write::WriteOptions};
pub use crate::ffi::{BlockBasedIndexType, DBCompressionType};
use crate::{
    Env, SliceTransform, ffi,
    statistics::{StatsLevel, Ticker},
};
use cxx::UniquePtr;
use std::pin::Pin;

/// Options for a DB and its column families.
///
/// The same type holds DB-wide options and per column family options. When
/// used for a column family, the DB-wide options are ignored.
pub struct Options {
    pub(crate) inner: UniquePtr<ffi::Options>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            inner: ffi::new_options(),
        }
    }
}

impl Clone for Options {
    /// Copies the options. Shared objects like the statistics, block cache
    /// and prefix extractor are shared by the copies.
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_options(&self.inner),
        }
    }
}

impl Options {
    fn pin_mut(&mut self) -> Pin<&mut ffi::Options> {
        self.inner.pin_mut()
    }

    /// Creates the DB if it is missing.
    pub fn create_if_missing(&mut self, value: bool) {
        self.pin_mut().set_create_if_missing(value);
    }

    /// Creates the column families that are missing when opening a DB.
    pub fn create_missing_column_families(&mut self, value: bool) {
        self.pin_mut().set_create_missing_column_families(value);
    }

    /// Sets the number of open files that can be used by the DB. -1 means
    /// unlimited.
    pub fn set_max_open_files(&mut self, value: i32) {
        self.pin_mut().set_max_open_files(value);
    }

    /// Sets the total size of WAL files after which the column families
    /// backed by the oldest WAL file are flushed. 0 means automatic.
    pub fn set_max_total_wal_size(&mut self, size: u64) {
        self.pin_mut().set_max_total_wal_size(size);
    }

    /// Sets the maximum number of concurrent flushes and compactions.
    pub fn set_max_background_jobs(&mut self, jobs: i32) {
        self.pin_mut().set_max_background_jobs(jobs);
    }

    /// Dumps statistics to the info log every `period` seconds. 0 disables
    /// dumping.
    pub fn set_stats_dump_period_sec(&mut self, period: u32) {
        self.pin_mut().set_stats_dump_period_sec(period);
    }

    pub fn set_env(&mut self, env: &Env) {
        self.pin_mut().set_env(&env.inner);
    }

    /// Enables statistics, which are shared by clones of these options and by
    /// the DBs opened with them.
    pub fn enable_statistics(&mut self) {
        self.pin_mut().enable_statistics();
    }

    /// Requires statistics to be enabled.
    pub fn set_statistics_level(&mut self, level: StatsLevel) {
        self.pin_mut().set_statistics_level(level);
    }

    /// Returns the value of a cumulative counter, or 0 if statistics are not
    /// enabled.
    pub fn get_ticker_count(&self, ticker: Ticker) -> u64 {
        self.inner.get_ticker_count(ticker)
    }

    pub fn set_compression_type(&mut self, compression: DBCompressionType) {
        self.pin_mut().set_compression_type(compression);
    }

    pub fn set_block_based_table_factory(&mut self, options: &BlockBasedOptions) {
        self.pin_mut().set_block_based_table_factory(&options.inner);
    }

    /// Marks SST files for compaction when, within any `window_size`
    /// consecutive entries, at least `num_dels_trigger` are deletions, or
    /// when the overall ratio of deletions is at least `deletion_ratio`.
    /// A ratio outside of (0, 1] disables the latter.
    pub fn add_compact_on_deletion_collector_factory(
        &mut self,
        window_size: usize,
        num_dels_trigger: usize,
        deletion_ratio: f64,
    ) {
        self.pin_mut().add_compact_on_deletion_collector_factory(
            window_size,
            num_dels_trigger,
            deletion_ratio,
        );
    }

    /// Slows down writes when the estimated bytes pending compaction exceed
    /// `limit`. 0 disables the limit.
    pub fn set_soft_pending_compaction_bytes_limit(&mut self, limit: u64) {
        self.pin_mut()
            .set_soft_pending_compaction_bytes_limit(limit);
    }

    /// Stops writes when the estimated bytes pending compaction exceed
    /// `limit`. 0 disables the limit.
    pub fn set_hard_pending_compaction_bytes_limit(&mut self, limit: u64) {
        self.pin_mut()
            .set_hard_pending_compaction_bytes_limit(limit);
    }

    /// Slows down writes when the number of L0 files reaches `num`.
    pub fn set_level_zero_slowdown_writes_trigger(&mut self, num: i32) {
        self.pin_mut().set_level_zero_slowdown_writes_trigger(num);
    }

    /// Stops writes when the number of L0 files reaches `num`.
    pub fn set_level_zero_stop_writes_trigger(&mut self, num: i32) {
        self.pin_mut().set_level_zero_stop_writes_trigger(num);
    }

    pub fn set_prefix_extractor(&mut self, transform: SliceTransform) {
        self.pin_mut().set_prefix_extractor(transform.inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_disabled() {
        let opts = Options::default();
        assert_eq!(opts.get_ticker_count(Ticker::BlockCacheMiss), 0);
    }

    #[test]
    fn invalid_ticker() {
        let mut opts = Options::default();
        opts.enable_statistics();
        assert_eq!(opts.get_ticker_count(Ticker { repr: u32::MAX }), 0);
    }

    #[test]
    fn invalid_enum_values_are_ignored() {
        let mut opts = Options::default();
        opts.enable_statistics();
        opts.set_statistics_level(StatsLevel { repr: u8::MAX });
        opts.set_compression_type(DBCompressionType { repr: u8::MAX });
        let mut table_opts = BlockBasedOptions::default();
        table_opts.set_index_type(BlockBasedIndexType { repr: u8::MAX });
    }

    #[test]
    fn setters() {
        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(2);
        env.set_high_priority_background_threads(1);

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(100);
        opts.set_max_total_wal_size(1 << 30);
        opts.set_max_background_jobs(4);
        opts.set_stats_dump_period_sec(0);
        opts.set_env(&env);
        opts.enable_statistics();
        opts.set_statistics_level(StatsLevel::ExceptHistogramOrTimers);
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.set_block_based_table_factory(&BlockBasedOptions::default());
        opts.add_compact_on_deletion_collector_factory(0, 0, 0.4);
        opts.set_soft_pending_compaction_bytes_limit(0);
        opts.set_hard_pending_compaction_bytes_limit(0);
        opts.set_level_zero_slowdown_writes_trigger(1000);
        opts.set_level_zero_stop_writes_trigger(2000);
        opts.set_prefix_extractor(SliceTransform::create("noop", |key| key, None));
        let _copy = opts.clone();
    }
}
