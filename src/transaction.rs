use crate::block::ClientID;
use crate::block_store::BlockStore;

pub struct Transaction {
    block_store: BlockStore,
    client: ClientID,
}

impl Transaction {
    pub fn new(client: ClientID) -> Self {
        Transaction {
            client,
            block_store: BlockStore::new(),
        }
    }

    pub fn insert(index: u32, content: String) {
        // invoke BlockStore.insert
    }

    pub fn delete(index: u32) {}

    pub fn update() {}

    fn split_node() {}
}
