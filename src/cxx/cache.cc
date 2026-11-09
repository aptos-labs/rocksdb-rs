#include "rocksdb-rs/src/cxx/cache.h"

namespace rocksdb_rs {

std::shared_ptr<rocksdb::Cache> new_hyper_clock_cache(
    size_t capacity, size_t estimated_entry_charge) {
  return rocksdb::HyperClockCacheOptions(capacity, estimated_entry_charge)
      .MakeSharedCache();
}

}  // namespace rocksdb_rs
