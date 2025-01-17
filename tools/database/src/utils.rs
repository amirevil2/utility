use std::fs;
use std::path::Path;

use anyhow::anyhow;
use strum::IntoEnumIterator;
use unc_primitives::hash::CryptoHash;
use unc_primitives::shard_layout::get_block_shard_uid;
use unc_store::flat::{store_helper, BlockInfo};
use unc_store::{DBCol, NodeStorage, ShardUId, Store};

pub(crate) fn open_rocksdb(
    home: &Path,
    mode: unc_store::Mode,
) -> anyhow::Result<unc_store::db::RocksDB> {
    let config = framework::config::Config::from_file_skip_validation(
        &home.join(framework::config::CONFIG_FILENAME),
    )?;
    let store_config = &config.store;
    let db_path = store_config.path.as_ref().cloned().unwrap_or_else(|| home.join("data"));
    let rocksdb =
        unc_store::db::RocksDB::open(&db_path, store_config, mode, unc_store::Temperature::Hot)?;
    Ok(rocksdb)
}

pub(crate) fn open_state_snapshot(home: &Path, mode: unc_store::Mode) -> anyhow::Result<Store> {
    let config = framework::config::Config::from_file_skip_validation(
        &home.join(framework::config::CONFIG_FILENAME),
    )?;
    let store_config = &config.store;
    let db_path = store_config.path.as_ref().cloned().unwrap_or_else(|| home.join("data"));

    let state_snapshot_dir = db_path.join("state_snapshot");
    let snapshots: Result<Vec<_>, _> = fs::read_dir(state_snapshot_dir)?.into_iter().collect();
    let snapshots = snapshots?;
    let &[snapshot_dir] = &snapshots.as_slice() else {
        return Err(anyhow!("found more than one snapshot"));
    };

    let path = snapshot_dir.path();
    println!("state snapshot path {path:?}");

    let opener = NodeStorage::opener(&path, false, &store_config, None);
    let storage = opener.open_in_mode(mode)?;
    let store = storage.get_hot_store();

    Ok(store)
}

pub(crate) fn resolve_column(col_name: &str) -> anyhow::Result<DBCol> {
    DBCol::iter()
        .filter(|db_col| <&str>::from(db_col) == col_name)
        .next()
        .ok_or_else(|| anyhow!("column {col_name} does not exist"))
}

pub fn flat_head_state_root(store: &Store, shard_uid: &ShardUId) -> CryptoHash {
    let chunk: unc_primitives::types::chunk_extra::ChunkExtra = store
        .get_ser(
            DBCol::ChunkExtra,
            &get_block_shard_uid(&flat_head(store, shard_uid).hash, shard_uid),
        )
        .unwrap()
        .unwrap();
    *chunk.state_root()
}

pub fn flat_head(store: &Store, shard_uid: &ShardUId) -> BlockInfo {
    match store_helper::get_flat_storage_status(store, *shard_uid).unwrap() {
        unc_store::flat::FlatStorageStatus::Ready(status) => status.flat_head,
        other => panic!("invalid flat storage status {other:?}"),
    }
}
