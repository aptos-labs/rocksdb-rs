#pragma once

#include <cstdint>
#include <memory>
#include <string>

#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb/cache.h"
#include "rocksdb/env.h"
#include "rocksdb/options.h"
#include "rocksdb/table.h"

namespace rocksdb_rs {

class BlockBasedOptions {
 public:
  void set_block_size(size_t size);
  void set_index_type(BlockBasedIndexType type);
  void set_partition_filters(bool value);
  void set_cache_index_and_filter_blocks(bool value);
  void set_pin_l0_filter_and_index_blocks_in_cache(bool value);
  void set_block_cache(const std::shared_ptr<rocksdb::Cache>& cache);
  void set_bloom_filter(double bits_per_key);
  void set_hybrid_ribbon_filter(double bloom_equivalent_bits_per_key,
                                int32_t bloom_before_level);
  void set_whole_key_filtering(bool value);

  rocksdb::BlockBasedTableOptions inner;
};

std::unique_ptr<BlockBasedOptions> new_block_based_options();

// Both DB-wide and column family options, like `rocksdb::Options`.
class Options {
 public:
  // DB options.
  void set_create_if_missing(bool value);
  void set_create_missing_column_families(bool value);
  void set_max_open_files(int32_t value);
  void set_max_total_wal_size(uint64_t value);
  void set_max_background_jobs(int32_t value);
  void set_stats_dump_period_sec(uint32_t value);
  void set_env(const std::shared_ptr<rocksdb::Env>& env);
  void enable_statistics();
  void set_statistics_level(StatsLevel level);
  uint64_t get_ticker_count(Ticker ticker) const;

  // Column family options.
  void set_compression_type(DBCompressionType type);
  void set_block_based_table_factory(const BlockBasedOptions& options);
  void add_compact_on_deletion_collector_factory(size_t window_size,
                                                 size_t num_dels_trigger,
                                                 double deletion_ratio);
  void set_soft_pending_compaction_bytes_limit(uint64_t value);
  void set_hard_pending_compaction_bytes_limit(uint64_t value);
  void set_level_zero_slowdown_writes_trigger(int32_t value);
  void set_level_zero_stop_writes_trigger(int32_t value);
  void set_prefix_extractor(rust::Box<RustSliceTransform> transform);

  rocksdb::Options inner;
  // Owns `inner.env`, which is a raw pointer.
  std::shared_ptr<rocksdb::Env> env;
};

std::unique_ptr<Options> new_options();
std::unique_ptr<Options> clone_options(const Options& options);

class ReadOptions {
 public:
  ReadOptions() = default;
  // `inner` points into this object.
  ReadOptions(const ReadOptions&) = delete;
  ReadOptions& operator=(const ReadOptions&) = delete;

  void fill_cache(bool value);
  void set_prefix_same_as_start(bool value);
  void set_total_order_seek(bool value);
  void set_max_skippable_internal_keys(uint64_t value);
  void set_iterate_upper_bound(Bytes key);

  rocksdb::ReadOptions inner;

 private:
  std::string upper_bound_;
  rocksdb::Slice upper_bound_slice_;
};

std::unique_ptr<ReadOptions> new_read_options();

class WriteOptions {
 public:
  void set_sync(bool value);

  rocksdb::WriteOptions inner;
};

std::unique_ptr<WriteOptions> new_write_options();

}  // namespace rocksdb_rs
