use crate::block::ClientID;
use crate::doc::Doc;
use std::sync::Arc;
use tokio::sync::Mutex;

// SyncTransaction is used to sync updates (insertion and deletion) among different clients
//
// IMPORTANT: SyncTransaction will take in a created Doc and modify its states

pub type Updates = Vec<Block>;

pub struct SyncTransaction {
    // local copy of the doc
    doc: Arc<Mutex<Doc>>,

    // unique identifier for a client
    client: ClientID,
}

impl SyncTransaction {
    pub fn new(client: ClientID, doc: Arc<Mutex<Doc>>) -> Self {
        SyncTransaction {
            client,
            doc,
        }
    }

    // apply its own updates on local copy
    pub fn update() {}

    // apply its own deletes on local copy
    pub fn delete() {}

    // update peers' modifications on local copy
    pub fn update_remote() {}

    // obtain a delete set from update_remote(), and apply peer deletions
    pub fn delete_remote() {}

    // takes in a vector clock, compare with its own vector clock, 
    // compute updates that need to be send
    fn compute_diff() {}

    // given a diff, consult the block store and find all the updates 
    // need to send to the counterpart
    fn construct_updates() {}

    // todo: rpc interface
    // request: doc name, vector clock
    // response: missing updates
    pub fn get_remote_updates() {}

    // todo: rpc interface, called when a new node joins the peer
    // request: the new node id
    // reponse: the entire state of the Doc (for the new peer to construct the whole doc)
    pub fn register_new_peer() {}
}
