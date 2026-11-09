#include "rocksdb-rs/src/cxx/common.h"

#include "rocksdb-rs/src/ffi.rs.h"

namespace rocksdb_rs {

namespace {

StatusCode to_status_code(rocksdb::Status::Code code) {
  using Code = rocksdb::Status::Code;
  switch (code) {
    case Code::kOk:
      return StatusCode::Ok;
    case Code::kNotFound:
      return StatusCode::NotFound;
    case Code::kCorruption:
      return StatusCode::Corruption;
    case Code::kNotSupported:
      return StatusCode::NotSupported;
    case Code::kInvalidArgument:
      return StatusCode::InvalidArgument;
    case Code::kIOError:
      return StatusCode::IOError;
    case Code::kMergeInProgress:
      return StatusCode::MergeInProgress;
    case Code::kIncomplete:
      return StatusCode::Incomplete;
    case Code::kShutdownInProgress:
      return StatusCode::ShutdownInProgress;
    case Code::kTimedOut:
      return StatusCode::TimedOut;
    case Code::kAborted:
      return StatusCode::Aborted;
    case Code::kBusy:
      return StatusCode::Busy;
    case Code::kExpired:
      return StatusCode::Expired;
    case Code::kTryAgain:
      return StatusCode::TryAgain;
    case Code::kCompactionTooLarge:
      return StatusCode::CompactionTooLarge;
    case Code::kColumnFamilyDropped:
      return StatusCode::ColumnFamilyDropped;
    case Code::kMaxCode:
      break;
  }
  return StatusCode::Unknown;
}

}  // namespace

void set_status(Status& out, const rocksdb::Status& s) {
  if (s.ok()) {
    return;
  }
  out.code = to_status_code(s.code());
  // Messages may embed paths or keys that are not valid UTF-8.
  out.message = rust::String::lossy(s.ToString());
}

}  // namespace rocksdb_rs
