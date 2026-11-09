#include "rocksdb-rs/src/cxx/env.h"

namespace rocksdb_rs {

std::shared_ptr<rocksdb::Env> default_env() {
  return std::shared_ptr<rocksdb::Env>(rocksdb::Env::Default(),
                                       [](rocksdb::Env*) {});
}

void set_low_priority_background_threads(
    const std::shared_ptr<rocksdb::Env>& env, int32_t num) {
  if (env) {
    env->SetBackgroundThreads(num, rocksdb::Env::Priority::LOW);
  }
}

void set_high_priority_background_threads(
    const std::shared_ptr<rocksdb::Env>& env, int32_t num) {
  if (env) {
    env->SetBackgroundThreads(num, rocksdb::Env::Priority::HIGH);
  }
}

}  // namespace rocksdb_rs
