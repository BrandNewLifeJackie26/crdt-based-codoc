use crate::{block_store::BlockStore, transaction::Transaction, ClientID};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Doc {
    name: String,
    client: ClientID,
    block_store: Arc<Mutex<BlockStore>>,
}

impl Doc {
    pub fn new(name: String, client: ClientID) -> Self {
        Doc {
            name,
            client,
            block_store: Arc::new(Mutex::new(BlockStore::new())),
        }
    }

    pub fn create_transaction(&self) -> Transaction {
        Transaction::new(self.client, self.block_store.clone())
    }
}
