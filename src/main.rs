use std::thread::sleep;
use std::time::Duration;

use aleo_std_storage::StorageMode;
use serde::{Deserialize, Serialize};
use snarkvm::ledger::store::helpers::rocksdb::internal::RocksDB;
use snarkvm::ledger::store::helpers::rocksdb::{BFTMap, BlockMap, DataMap, Database, MapID};
use snarkvm::ledger::store::helpers::Map;
use snarkvm::prelude::TestRng;
use snarkvm::{console::program::Rng, ledger::store::helpers::MapRead};
use tokio::task::JoinSet;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter::EnvFilter, util::SubscriberInitExt};

use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const NUM_TASKS: usize = 100;
const BLOCK_SIZE: usize = 1024 * 1024;
const MAX_SLEEP_TIME: u64 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct BigData {
    data: Vec<u8>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 50)]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let mut set = JoinSet::new();
    info!("starting readers");
    for _ in 1..NUM_TASKS {
        set.spawn(async {
            let storage_mode = StorageMode::Development(1);
            let transmissions: DataMap<u64, BigData> =
                RocksDB::open_map(0, storage_mode.clone(), MapID::BFT(BFTMap::Transmissions))
                    .unwrap();
            let mut rng = TestRng::default();
            info!("reader started");
            loop {
                transmissions.iter_pending().for_each(|(k, v)| {
                    info!("pending: key: {}, value: {:?}", k, v);
                });
                transmissions.iter_confirmed().for_each(|(k, v)| {
                    info!("confirmed: key: {}, value: {:?}", k, v);
                });
                let sleep_time = rng.gen_range(200..MAX_SLEEP_TIME);
                sleep(Duration::from_millis(sleep_time));
            }
        });
    }
    info!("starting writers");
    for _ in 1..NUM_TASKS {
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
                        data: (0..BLOCK_SIZE).map(|_| rng.gen()).collect(),
                    };
                    transmissions.insert(rng.gen(), big_data.clone()).unwrap();
                    let big_data = BigData {
                        data: (0..BLOCK_SIZE).map(|_| rng.gen()).collect(),
                    };
                    blocks.insert(rng.gen(), big_data).unwrap();
                }
                let sleep_time = rng.gen_range(200..MAX_SLEEP_TIME);
                sleep(Duration::from_millis(sleep_time));
            }
        });
    }
    set.join_next().await;
}
