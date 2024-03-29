use std::thread::sleep;
use std::time::Duration;

use aleo_std_storage::StorageMode;
use snarkvm::console::program::Rng;
use snarkvm::ledger::store::helpers::rocksdb::internal::RocksDB;
use snarkvm::ledger::store::helpers::rocksdb::{BFTMap, BlockMap, DataMap, Database, MapID};
use snarkvm::ledger::store::helpers::Map;
use snarkvm::prelude::TestRng;

use tikv_jemallocator::Jemalloc;
use tokio::task::JoinSet;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const NUM_THREADS: usize = 10;
const SLEEP_TIME: u64 = 10;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();
    for _ in 1..NUM_THREADS {
        set.spawn(async {
            let storage_mode = StorageMode::Development(1);
            let transmissions: DataMap<u64, u64> =
                RocksDB::open_map(0, storage_mode.clone(), MapID::BFT(BFTMap::Transmissions))
                    .unwrap();
            let blocks: DataMap<u64, u64> =
                RocksDB::open_map(0, storage_mode, MapID::Block(BlockMap::StateRoot)).unwrap();
            let mut rng = TestRng::default();
            loop {
                for _ in 0..1000 {
                    transmissions.insert(rng.gen(), rng.gen()).unwrap();
                    blocks.insert(rng.gen(), rng.gen()).unwrap();
                }
                sleep(Duration::from_millis(SLEEP_TIME));
            }
        });
    }
    set.join_next().await;
}
