use crate::block_store::BlockStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Doc {
    name: String,
    block_store: Arc<Mutex<BlockStore>>,
}

impl Doc {
    pub fn new(name: String) -> Self {
        Doc {
            name,
            block_store: Arc::new(Mutex::new(BlockStore::new())),
        }
    }
}
