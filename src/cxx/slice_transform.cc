#include "rocksdb-rs/src/cxx/slice_transform.h"

#include <algorithm>
#include <string>

#include "rocksdb-rs/src/ffi.rs.h"

namespace rocksdb_rs {

namespace {

class RustSliceTransformAdapter : public rocksdb::SliceTransform {
 public:
  explicit RustSliceTransformAdapter(rust::Box<RustSliceTransform> transform)
      : transform_(std::move(transform)),
        name_(std::string(transform_->name())) {}

  const char* Name() const override { return name_.c_str(); }

  rocksdb::Slice Transform(const rocksdb::Slice& key) const override {
    KeyRange range = transform_->transform(to_bytes(key));
    // Rust checks the range; clamp anyway rather than trust it blindly.
    size_t start = std::min(range.start, key.size());
    size_t len = std::min(range.len, key.size() - start);
    return rocksdb::Slice(key.data() + start, len);
  }

  bool InDomain(const rocksdb::Slice& key) const override {
    return transform_->in_domain(to_bytes(key));
  }

 private:
  rust::Box<RustSliceTransform> transform_;
  // `Name()` must return a stable C string.
  std::string name_;
};

}  // namespace

std::shared_ptr<const rocksdb::SliceTransform> new_slice_transform(
    rust::Box<RustSliceTransform> transform) {
  return std::make_shared<RustSliceTransformAdapter>(std::move(transform));
}

}  // namespace rocksdb_rs
