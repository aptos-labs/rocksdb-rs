#include "rocksdb-rs/src/cxx/db.h"

#include <string>

#include "rocksdb-rs/src/ffi.rs.h"
#include "rocksdb/utilities/checkpoint.h"

namespace rocksdb_rs {

Db::Db(std::unique_ptr<rocksdb::DB> db,
       std::vector<rocksdb::ColumnFamilyHandle*> handles,
       std::shared_ptr<rocksdb::Env> env)
    : db_(std::move(db)), env_(std::move(env)) {
  column_families_.reserve(handles.size());
  for (auto* handle : handles) {
    column_families_.push_back(
        std::make_unique<ColumnFamily>(db_.get(), handle));
  }
}

Db::~Db() {
  // Handles must be released before the DB, and the DB before its `Env`.
  column_families_.clear();
  db_.reset();
}

size_t Db::num_column_families() const { return column_families_.size(); }

const ColumnFamily& Db::column_family(size_t index) const {
  return *column_families_.at(index);
}

bool Db::check_owner(const ColumnFamily& cf, Status& status) const {
  if (cf.db() == db_.get()) {
    return true;
  }
  set_status(status, rocksdb::Status::InvalidArgument(
                         "column family belongs to another DB"));
  return false;
}

bool Db::get(const ReadOptions& options, const ColumnFamily& cf, Bytes key,
             rust::Vec<uint8_t>& value, Status& status) const {
  if (!check_owner(cf, status)) {
    return false;
  }
  rocksdb::PinnableSlice pinned;
  auto s = db_->Get(options.inner, cf.handle(), to_slice(key), &pinned);
  if (s.IsNotFound()) {
    return false;
  }
  set_status(status, s);
  if (!s.ok()) {
    return false;
  }
  extend_from_slice(value, to_bytes(pinned));
  return true;
}

void Db::put(const WriteOptions& options, const ColumnFamily& cf, Bytes key,
             Bytes value, Status& status) const {
  if (check_owner(cf, status)) {
    set_status(status, db_->Put(options.inner, cf.handle(), to_slice(key),
                                to_slice(value)));
  }
}

void Db::del(const WriteOptions& options, const ColumnFamily& cf, Bytes key,
             Status& status) const {
  if (check_owner(cf, status)) {
    set_status(status, db_->Delete(options.inner, cf.handle(), to_slice(key)));
  }
}

void Db::write(const WriteOptions& options, WriteBatch& batch,
               Status& status) const {
  set_status(status, db_->Write(options.inner, &batch.inner));
}

void Db::flush(const ColumnFamily& cf, Status& status) const {
  if (check_owner(cf, status)) {
    set_status(status, db_->Flush(rocksdb::FlushOptions(), cf.handle()));
  }
}

bool Db::get_int_property(const ColumnFamily& cf, rust::Str name,
                          uint64_t& value, Status& status) const {
  if (!check_owner(cf, status)) {
    return false;
  }
  return db_->GetIntProperty(cf.handle(),
                             rocksdb::Slice(name.data(), name.size()), &value);
}

std::unique_ptr<DbIterator> Db::new_iterator(
    std::unique_ptr<ReadOptions> options, const ColumnFamily& cf) const {
  if (!options) {
    options = std::make_unique<ReadOptions>();
  }
  std::unique_ptr<rocksdb::Iterator> iter;
  if (cf.db() == db_.get()) {
    iter.reset(db_->NewIterator(options->inner, cf.handle()));
  } else {
    iter.reset(rocksdb::NewErrorIterator(rocksdb::Status::InvalidArgument(
        "column family belongs to another DB")));
  }
  return std::make_unique<DbIterator>(std::move(options), std::move(iter));
}

void Db::create_checkpoint(Bytes path, Status& status) const {
  rocksdb::Checkpoint* raw = nullptr;
  auto s = rocksdb::Checkpoint::Create(db_.get(), &raw);
  std::unique_ptr<rocksdb::Checkpoint> checkpoint(raw);
  if (s.ok()) {
    s = checkpoint->CreateCheckpoint(to_string(path));
  }
  set_status(status, s);
}

namespace {

rocksdb::Status open(const rocksdb::DBOptions& options, const std::string& path,
                     const std::vector<rocksdb::ColumnFamilyDescriptor>& cfs,
                     OpenMode mode, Bytes secondary_path,
                     bool error_if_wal_file_exists,
                     std::vector<rocksdb::ColumnFamilyHandle*>& handles,
                     std::unique_ptr<rocksdb::DB>& db) {
  switch (mode) {
    case OpenMode::ReadWrite:
      return rocksdb::DB::Open(options, path, cfs, &handles, &db);
    case OpenMode::ReadOnly:
      return rocksdb::DB::OpenForReadOnly(options, path, cfs, &handles, &db,
                                          error_if_wal_file_exists);
    case OpenMode::Secondary:
      return rocksdb::DB::OpenAsSecondary(
          options, path, to_string(secondary_path), cfs, &handles, &db);
  }
  return rocksdb::Status::InvalidArgument("unknown open mode");
}

}  // namespace

std::unique_ptr<Db> open_db(const Options& options, Bytes path,
                            const ColumnFamilyDescriptors& column_families,
                            OpenMode mode, Bytes secondary_path,
                            bool error_if_wal_file_exists, Status& status) {
  std::vector<rocksdb::ColumnFamilyHandle*> handles;
  std::unique_ptr<rocksdb::DB> db;
  auto s = open(rocksdb::DBOptions(options.inner), to_string(path),
                column_families.inner, mode, secondary_path,
                error_if_wal_file_exists, handles, db);
  if (!s.ok()) {
    set_status(status, s);
    return nullptr;
  }

  auto result =
      std::make_unique<Db>(std::move(db), std::move(handles), options.env);
  // Rust looks up column families by their position in `column_families`.
  if (result->num_column_families() != column_families.inner.size()) {
    set_status(status, rocksdb::Status::Corruption(
                           "unexpected number of column family handles"));
    return nullptr;
  }
  return result;
}

rust::Vec<rust::String> list_column_families(const Options& options, Bytes path,
                                             Status& status) {
  std::vector<std::string> names;
  auto s = rocksdb::DB::ListColumnFamilies(rocksdb::DBOptions(options.inner),
                                           to_string(path), &names);
  set_status(status, s);

  rust::Vec<rust::String> result;
  if (s.ok()) {
    result.reserve(names.size());
    for (const auto& name : names) {
      result.push_back(rust::String::lossy(name));
    }
  }
  return result;
}

}  // namespace rocksdb_rs
