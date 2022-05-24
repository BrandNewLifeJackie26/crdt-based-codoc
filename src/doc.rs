use crate::utils::{ClientID, Peer, Updates};
use crate::{block::Content, block_store::BlockStore};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

// VectorClock represents the latest clocks of all clients,
// it is used during synchronization to find the missing changes

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorClock {
    pub clock_map: HashMap<ClientID, u32>,
}

impl VectorClock {
    pub fn from() -> VectorClock {
        todo!()
    }
}

// Doc is the collaborative edited document,
// it is owned by client, block_store is the real storage of all elements inside the doc.
//
// Doc also stores some pending updates to avoid out-of-order updates
//
// IMPORTANT: Doc = block_store + state
pub struct Doc {
    pub name: String,
    pub client: ClientID,
    pub block_store: Arc<Mutex<BlockStore>>,

    // list of peers that are collaborately editing the same doc
    pub peers: Vec<Peer>,

    pub pending_updates: Updates,

    // TODO: states: vector clock, pending updates, delete set, etc.
    pub vector_clock: VectorClock,
}

impl Doc {
    pub fn new(name: String, client: ClientID) -> Self {
        Doc {
            name,
            client,
            block_store: Arc::new(Mutex::new(BlockStore::new())),
            peers: vec![],
            pending_updates: vec![],
            vector_clock: VectorClock {
                clock_map: HashMap::new(),
            },
        }
    }

    /* Local operations */
    // Insert the content into pos in BlockStore
    pub async fn insert(&self, content: Content, pos: u32) {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        (*store_lock).insert(content.clone(), pos);
        // TODO: update vector clock
        todo!()
    }

    // Delete the content of length len from pos
    pub async fn delete(&self, pos: u32, len: u32) {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        (*store_lock).delete(pos, len);
        // TODO: update vector clock
        todo!()
    }
}
