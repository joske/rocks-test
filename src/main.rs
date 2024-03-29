use std::thread::sleep;
use std::time::Duration;

use aleo_std_storage::StorageMode;
use serde::{Deserialize, Serialize};
use snarkvm::console::program::Rng;
use snarkvm::ledger::store::helpers::rocksdb::internal::RocksDB;
use snarkvm::ledger::store::helpers::rocksdb::{BFTMap, BlockMap, DataMap, Database, MapID};
use snarkvm::ledger::store::helpers::Map;
use snarkvm::prelude::TestRng;
use tokio::task::JoinSet;

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const NUM_THREADS: usize = 100;
const MAX_SLEEP_TIME: u64 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct BigData {
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();
    for _ in 1..NUM_THREADS {
        set.spawn(async {
            let storage_mode = StorageMode::Development(1);
            let transmissions: DataMap<u64, BigData> =
                RocksDB::open_map(0, storage_mode.clone(), MapID::BFT(BFTMap::Transmissions))
                    .unwrap();
            let blocks: DataMap<u64, BigData> =
                RocksDB::open_map(0, storage_mode, MapID::Block(BlockMap::StateRoot)).unwrap();
            let mut rng = TestRng::default();
            loop {
                for _ in 0..1000 {
                    let big_data = BigData {
                        data: (0..1000).map(|_| rng.gen()).collect(),
                    };
                    transmissions.insert(rng.gen(), big_data.clone()).unwrap();
                    blocks.insert(rng.gen(), big_data).unwrap();
                }
                let sleep_time = rng.gen_range(200..MAX_SLEEP_TIME);
                sleep(Duration::from_millis(sleep_time));
            }
        });
    }
    set.join_next().await;
}
