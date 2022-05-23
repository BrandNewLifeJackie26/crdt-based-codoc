use crate::block::ClientID;
use crate::doc::Doc;
use std::sync::Arc;
use tokio::sync::Mutex;

// SyncTransaction is used to sync updates (insertion and deletion) among different clients
//
// IMPORTANT: SyncTransaction will take in a created Doc and modify its states
pub struct SyncTransaction {
    doc: Arc<Mutex<Doc>>,
    client: ClientID,
}

impl SyncTransaction {
    pub fn new(client: ClientID, doc: Arc<Mutex<Doc>>) -> Self {
        SyncTransaction {
            client,
            doc,
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
