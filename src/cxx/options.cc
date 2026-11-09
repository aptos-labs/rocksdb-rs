#include "rocksdb-rs/src/cxx/options.h"

#include <optional>

#include "rocksdb-rs/src/cxx/slice_transform.h"
#include "rocksdb-rs/src/ffi.rs.h"
#include "rocksdb/filter_policy.h"
#include "rocksdb/statistics.h"
#include "rocksdb/utilities/table_properties_collectors.h"

namespace rocksdb_rs {

namespace {

// Shared enums can hold any value of the underlying type, so every mapping
// handles values outside of the declared variants.

std::optional<rocksdb::CompressionType> to_rocksdb(DBCompressionType type) {
  switch (type) {
    case DBCompressionType::None:
      return rocksdb::kNoCompression;
    case DBCompressionType::Lz4:
      return rocksdb::kLZ4Compression;
    case DBCompressionType::Lz4hc:
      return rocksdb::kLZ4HCCompression;
  }
  return std::nullopt;
}

std::optional<rocksdb::BlockBasedTableOptions::IndexType> to_rocksdb(
    BlockBasedIndexType type) {
  using IndexType = rocksdb::BlockBasedTableOptions::IndexType;
  switch (type) {
    case BlockBasedIndexType::BinarySearch:
      return IndexType::kBinarySearch;
    case BlockBasedIndexType::HashSearch:
      return IndexType::kHashSearch;
    case BlockBasedIndexType::TwoLevelIndexSearch:
      return IndexType::kTwoLevelIndexSearch;
  }
  return std::nullopt;
}

std::optional<rocksdb::StatsLevel> to_rocksdb(StatsLevel level) {
  switch (level) {
    case StatsLevel::DisableAll:
      return rocksdb::StatsLevel::kDisableAll;
    case StatsLevel::ExceptHistogramOrTimers:
      return rocksdb::StatsLevel::kExceptHistogramOrTimers;
    case StatsLevel::ExceptTimers:
      return rocksdb::StatsLevel::kExceptTimers;
    case StatsLevel::ExceptDetailedTimers:
      return rocksdb::StatsLevel::kExceptDetailedTimers;
    case StatsLevel::ExceptTimeForMutex:
      return rocksdb::StatsLevel::kExceptTimeForMutex;
    case StatsLevel::All:
      return rocksdb::StatsLevel::kAll;
  }
  return std::nullopt;
}

std::optional<rocksdb::Tickers> to_rocksdb(Ticker ticker) {
  switch (ticker) {
    case Ticker::BlockCacheMiss:
      return rocksdb::BLOCK_CACHE_MISS;
    case Ticker::BlockCacheHit:
      return rocksdb::BLOCK_CACHE_HIT;
    case Ticker::BlockCacheIndexMiss:
      return rocksdb::BLOCK_CACHE_INDEX_MISS;
    case Ticker::BlockCacheIndexHit:
      return rocksdb::BLOCK_CACHE_INDEX_HIT;
    case Ticker::BlockCacheFilterMiss:
      return rocksdb::BLOCK_CACHE_FILTER_MISS;
    case Ticker::BlockCacheFilterHit:
      return rocksdb::BLOCK_CACHE_FILTER_HIT;
    case Ticker::BlockCacheDataMiss:
      return rocksdb::BLOCK_CACHE_DATA_MISS;
    case Ticker::BlockCacheDataHit:
      return rocksdb::BLOCK_CACHE_DATA_HIT;
    case Ticker::BloomFilterUseful:
      return rocksdb::BLOOM_FILTER_USEFUL;
    case Ticker::BloomFilterFullPositive:
      return rocksdb::BLOOM_FILTER_FULL_POSITIVE;
    case Ticker::BloomFilterFullTruePositive:
      return rocksdb::BLOOM_FILTER_FULL_TRUE_POSITIVE;
    case Ticker::BloomFilterPrefixChecked:
      return rocksdb::BLOOM_FILTER_PREFIX_CHECKED;
    case Ticker::BloomFilterPrefixUseful:
      return rocksdb::BLOOM_FILTER_PREFIX_USEFUL;
    case Ticker::BloomFilterPrefixTruePositive:
      return rocksdb::BLOOM_FILTER_PREFIX_TRUE_POSITIVE;
    case Ticker::StallMicros:
      return rocksdb::STALL_MICROS;
    case Ticker::CompactReadBytes:
      return rocksdb::COMPACT_READ_BYTES;
    case Ticker::CompactWriteBytes:
      return rocksdb::COMPACT_WRITE_BYTES;
    case Ticker::FlushWriteBytes:
      return rocksdb::FLUSH_WRITE_BYTES;
    case Ticker::LastLevelSeekFiltered:
      return rocksdb::LAST_LEVEL_SEEK_FILTERED;
    case Ticker::LastLevelSeekFilterMatch:
      return rocksdb::LAST_LEVEL_SEEK_FILTER_MATCH;
    case Ticker::LastLevelSeekData:
      return rocksdb::LAST_LEVEL_SEEK_DATA;
    case Ticker::LastLevelSeekDataUsefulNoFilter:
      return rocksdb::LAST_LEVEL_SEEK_DATA_USEFUL_NO_FILTER;
    case Ticker::LastLevelSeekDataUsefulFilterMatch:
      return rocksdb::LAST_LEVEL_SEEK_DATA_USEFUL_FILTER_MATCH;
    case Ticker::NonLastLevelSeekFiltered:
      return rocksdb::NON_LAST_LEVEL_SEEK_FILTERED;
    case Ticker::NonLastLevelSeekFilterMatch:
      return rocksdb::NON_LAST_LEVEL_SEEK_FILTER_MATCH;
    case Ticker::NonLastLevelSeekData:
      return rocksdb::NON_LAST_LEVEL_SEEK_DATA;
    case Ticker::NonLastLevelSeekDataUsefulNoFilter:
      return rocksdb::NON_LAST_LEVEL_SEEK_DATA_USEFUL_NO_FILTER;
    case Ticker::NonLastLevelSeekDataUsefulFilterMatch:
      return rocksdb::NON_LAST_LEVEL_SEEK_DATA_USEFUL_FILTER_MATCH;
  }
  return std::nullopt;
}

}  // namespace

// BlockBasedOptions

void BlockBasedOptions::set_block_size(size_t size) { inner.block_size = size; }

void BlockBasedOptions::set_index_type(BlockBasedIndexType type) {
  if (auto index_type = to_rocksdb(type)) {
    inner.index_type = *index_type;
  }
}

void BlockBasedOptions::set_partition_filters(bool value) {
  inner.partition_filters = value;
}

void BlockBasedOptions::set_cache_index_and_filter_blocks(bool value) {
  inner.cache_index_and_filter_blocks = value;
}

void BlockBasedOptions::set_pin_l0_filter_and_index_blocks_in_cache(
    bool value) {
  inner.pin_l0_filter_and_index_blocks_in_cache = value;
}

void BlockBasedOptions::set_block_cache(
    const std::shared_ptr<rocksdb::Cache>& cache) {
  inner.block_cache = cache;
}

void BlockBasedOptions::set_bloom_filter(double bits_per_key) {
  inner.filter_policy.reset(rocksdb::NewBloomFilterPolicy(bits_per_key));
}

void BlockBasedOptions::set_hybrid_ribbon_filter(
    double bloom_equivalent_bits_per_key, int32_t bloom_before_level) {
  inner.filter_policy.reset(rocksdb::NewRibbonFilterPolicy(
      bloom_equivalent_bits_per_key, bloom_before_level));
}

void BlockBasedOptions::set_whole_key_filtering(bool value) {
  inner.whole_key_filtering = value;
}

std::unique_ptr<BlockBasedOptions> new_block_based_options() {
  return std::make_unique<BlockBasedOptions>();
}

// Options

void Options::set_create_if_missing(bool value) {
  inner.create_if_missing = value;
}

void Options::set_create_missing_column_families(bool value) {
  inner.create_missing_column_families = value;
}

void Options::set_max_open_files(int32_t value) {
  inner.max_open_files = value;
}

void Options::set_max_total_wal_size(uint64_t value) {
  inner.max_total_wal_size = value;
}

void Options::set_max_background_jobs(int32_t value) {
  inner.max_background_jobs = value;
}

void Options::set_stats_dump_period_sec(uint32_t value) {
  inner.stats_dump_period_sec = value;
}

void Options::set_env(const std::shared_ptr<rocksdb::Env>& env) {
  this->env = env;
  inner.env = env ? env.get() : rocksdb::Env::Default();
}

void Options::enable_statistics() {
  inner.statistics = rocksdb::CreateDBStatistics();
}

void Options::set_statistics_level(StatsLevel level) {
  auto stats_level = to_rocksdb(level);
  if (inner.statistics && stats_level) {
    inner.statistics->set_stats_level(*stats_level);
  }
}

uint64_t Options::get_ticker_count(Ticker ticker) const {
  auto ticker_type = to_rocksdb(ticker);
  if (!inner.statistics || !ticker_type) {
    return 0;
  }
  return inner.statistics->getTickerCount(*ticker_type);
}

void Options::set_compression_type(DBCompressionType type) {
  if (auto compression = to_rocksdb(type)) {
    inner.compression = *compression;
  }
}

void Options::set_block_based_table_factory(const BlockBasedOptions& options) {
  inner.table_factory.reset(rocksdb::NewBlockBasedTableFactory(options.inner));
}

void Options::add_compact_on_deletion_collector_factory(size_t window_size,
                                                        size_t num_dels_trigger,
                                                        double deletion_ratio) {
  inner.table_properties_collector_factories.emplace_back(
      rocksdb::NewCompactOnDeletionCollectorFactory(
          window_size, num_dels_trigger, deletion_ratio));
}

void Options::set_soft_pending_compaction_bytes_limit(uint64_t value) {
  inner.soft_pending_compaction_bytes_limit = value;
}

void Options::set_hard_pending_compaction_bytes_limit(uint64_t value) {
  inner.hard_pending_compaction_bytes_limit = value;
}

void Options::set_level_zero_slowdown_writes_trigger(int32_t value) {
  inner.level0_slowdown_writes_trigger = value;
}

void Options::set_level_zero_stop_writes_trigger(int32_t value) {
  inner.level0_stop_writes_trigger = value;
}

void Options::set_prefix_extractor(rust::Box<RustSliceTransform> transform) {
  inner.prefix_extractor = new_slice_transform(std::move(transform));
}

std::unique_ptr<Options> new_options() { return std::make_unique<Options>(); }

std::unique_ptr<Options> clone_options(const Options& options) {
  return std::make_unique<Options>(options);
}

// ReadOptions

void ReadOptions::fill_cache(bool value) { inner.fill_cache = value; }

void ReadOptions::set_prefix_same_as_start(bool value) {
  inner.prefix_same_as_start = value;
}

void ReadOptions::set_total_order_seek(bool value) {
  inner.total_order_seek = value;
}

void ReadOptions::set_max_skippable_internal_keys(uint64_t value) {
  inner.max_skippable_internal_keys = value;
}

void ReadOptions::set_iterate_upper_bound(Bytes key) {
  upper_bound_ = to_string(key);
  upper_bound_slice_ = rocksdb::Slice(upper_bound_);
  inner.iterate_upper_bound = &upper_bound_slice_;
}

std::unique_ptr<ReadOptions> new_read_options() {
  return std::make_unique<ReadOptions>();
}

// WriteOptions

void WriteOptions::set_sync(bool value) { inner.sync = value; }

std::unique_ptr<WriteOptions> new_write_options() {
  return std::make_unique<WriteOptions>();
}

}  // namespace rocksdb_rs
