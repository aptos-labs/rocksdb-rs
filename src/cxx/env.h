#pragma once

#include <cstdint>
#include <memory>

#include "rocksdb/env.h"

namespace rocksdb_rs {

// The process-wide default `Env`. It is never destroyed.
std::shared_ptr<rocksdb::Env> default_env();

// `rocksdb::Env` is internally synchronized, so these take shared references.
void set_low_priority_background_threads(
    const std::shared_ptr<rocksdb::Env>& env, int32_t num);
void set_high_priority_background_threads(
    const std::shared_ptr<rocksdb::Env>& env, int32_t num);

}  // namespace rocksdb_rs
