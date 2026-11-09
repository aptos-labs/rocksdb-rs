#pragma once

#include <memory>

#include "rocksdb/cache.h"

namespace rocksdb_rs {

std::shared_ptr<rocksdb::Cache> new_hyper_clock_cache(
    size_t capacity, size_t estimated_entry_charge);

}  // namespace rocksdb_rs
