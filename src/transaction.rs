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

    pub fn delete() {}

    // takes in a vector clock, compare with its own vector clock, 
    // compute updates that need to be send
    fn compute_diff() {}

    // given a diff, consult the block store and find all the updates 
    // need to send to the counterpart
    fn construct_updates() {}

}
