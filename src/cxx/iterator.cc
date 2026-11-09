#include "rocksdb-rs/src/cxx/iterator.h"

#include "rocksdb-rs/src/ffi.rs.h"

namespace rocksdb_rs {

bool DbIterator::valid() const { return iter_->Valid(); }

void DbIterator::seek_to_first() { iter_->SeekToFirst(); }

void DbIterator::seek_to_last() { iter_->SeekToLast(); }

void DbIterator::seek(Bytes key) { iter_->Seek(to_slice(key)); }

void DbIterator::seek_for_prev(Bytes key) { iter_->SeekForPrev(to_slice(key)); }

void DbIterator::next() {
  if (iter_->Valid()) {
    iter_->Next();
  }
}

void DbIterator::prev() {
  if (iter_->Valid()) {
    iter_->Prev();
  }
}

Bytes DbIterator::key() const {
  return iter_->Valid() ? to_bytes(iter_->key()) : Bytes();
}

Bytes DbIterator::value() const {
  return iter_->Valid() ? to_bytes(iter_->value()) : Bytes();
}

void DbIterator::status(Status& status) const {
  set_status(status, iter_->status());
}

}  // namespace rocksdb_rs
