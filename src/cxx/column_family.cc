#include "rocksdb-rs/src/cxx/column_family.h"

#include <string>

namespace rocksdb_rs {

void ColumnFamilyDescriptors::add(rust::Str name, const Options& options) {
  inner.emplace_back(std::string(name),
                     rocksdb::ColumnFamilyOptions(options.inner));
}

std::unique_ptr<ColumnFamilyDescriptors> new_column_family_descriptors() {
  return std::make_unique<ColumnFamilyDescriptors>();
}

}  // namespace rocksdb_rs
