//! Statistics collected by RocksDB once enabled with
//! [`Options::enable_statistics`](crate::Options::enable_statistics).

pub use crate::ffi::{StatsLevel, Ticker};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DB, test_utils::create_options};
    use tempfile::TempDir;

    #[test]
    fn tickers() {
        let mut opts = create_options();
        opts.enable_statistics();
        // Clones share the statistics.
        let stats = opts.clone();
        let dir = TempDir::new().unwrap();
        let db = DB::open(&opts, dir.path()).unwrap();
        db.put(b"k", b"v").unwrap();
        db.flush().unwrap();
        assert!(stats.get_ticker_count(Ticker::FlushWriteBytes) > 0);
        assert_eq!(stats.get_ticker_count(Ticker::BlockCacheDataMiss), 0);

        db.get(b"k").unwrap();
        assert_eq!(stats.get_ticker_count(Ticker::BlockCacheDataMiss), 1);
        assert_eq!(stats.get_ticker_count(Ticker::BlockCacheMiss), 1);
        db.get(b"k").unwrap();
        assert_eq!(stats.get_ticker_count(Ticker::BlockCacheDataHit), 1);
        assert_eq!(stats.get_ticker_count(Ticker::BlockCacheHit), 1);
        assert_eq!(opts.get_ticker_count(Ticker::BlockCacheHit), 1);
    }

    #[test]
    fn stats_level() {
        let collected = |level| {
            let mut opts = create_options();
            opts.enable_statistics();
            opts.set_statistics_level(level);
            let dir = TempDir::new().unwrap();
            let db = DB::open(&opts, dir.path()).unwrap();
            db.put(b"k", b"v").unwrap();
            db.flush().unwrap();
            db.get(b"k").unwrap();
            opts.get_ticker_count(Ticker::BlockCacheDataMiss)
        };
        assert_eq!(collected(StatsLevel::DisableAll), 0);
        assert_eq!(collected(StatsLevel::ExceptHistogramOrTimers), 1);
        assert_eq!(collected(StatsLevel::All), 1);
    }
}
