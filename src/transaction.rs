use crate::block::ClientID;
use crate::block_store::BlockStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Transaction {
    block_store: Arc<Mutex<BlockStore>>,
    client: ClientID,
}

impl Transaction {
    pub fn new(client: ClientID, block_store: Arc<Mutex<BlockStore>>) -> Self {
        Transaction {
            client,
            block_store,
        }
    }

    pub fn update() {}
}
