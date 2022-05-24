use crate::utils::{ClientID, Peer, Updates};
use crate::{block::Content, block_store::BlockStore, Block, BlockID};
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

    fn increment(&mut self, client: ClientID) {
        self.clock_map
            .insert(
                client,
                self.clock_map.get(&client).cloned().unwrap_or(0) + 1,
            )
            .unwrap();
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
    pub async fn insert_remote(&mut self, update: Updates) {
        // let store = self.block_store.clone();
        // let mut store_lock = store.lock().await;
        // (*store_lock).insert(self.client, content.clone(), pos);
        // TODO: update vector clock
        // todo!()
    }

    // Insert the content into pos in BlockStore
    // TODO: Arc<Mutex<BlockList>>
    pub async fn insert_local(&mut self, content: Content, pos: u32) {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        // Find the correct block to insert
        let mut i = 0 as usize;
        let mut curr = 0;

        while curr < pos && i < (*store_lock).total_store.list.len() {
            // TODO: what if pos < curr and the loop already break?
            if !(*store_lock).total_store.list[i].is_deleted {
                curr += (*store_lock).total_store.list[i].content.content.len() as u32;
            }
            i += 1;
        }

        // Create a new block and insert it to total_store
        let mut new_block = Block {
            id: BlockID {
                client: self.client,
                clock: store_lock.kv_store.get(&self.client).unwrap().list.len() as u32 + 1,
            },
            left_origin: None,
            right_origin: None,
            is_deleted: false,
            content: content.clone(),
        };

        // TODO:
        if i == (*store_lock).total_store.list.len() {
            // Append to the end
            if i > 0 {
                let left_id = Some((*store_lock).total_store.list[i - 1].id.clone());
                new_block.left_origin = left_id.clone();
                (*store_lock).insert(new_block, left_id);
            } else {
                (*store_lock).insert(new_block, None);
            }
        } else if curr == pos {
            // Insert to i-th position in total_store
            let left_id = Some((*store_lock).total_store.list[i - 1].id.clone());
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some((*store_lock).total_store.list[i].id.clone());
            (*store_lock).insert(new_block, left_id);
        } else {
            // Have to split total_store[i-1]
            let left_id = Some((*store_lock).total_store.list[i - 1].id.clone());
            let left_content_len = (*store_lock).total_store.list.len();
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some(BlockID::new(
                left_id.clone().unwrap().client,
                left_id.clone().unwrap().clock + left_content_len as u32 - (curr - pos),
            ));
            (*store_lock).insert(new_block, left_id);
        }

        // Update vector clock
        self.vector_clock.increment(self.client);
    }

    // Delete the content of length len from pos
    pub async fn delete(&mut self, pos: u32, len: u32) {}
    pub async fn delete_remote(&mut self, update: Updates) {
        // let store = self.block_store.clone();
        // let mut store_lock = store.lock().await;

        // // Find the correct block to delete
        // // The block may need to be splitted
        // let mut i = 0 as usize;
        // let mut curr = 0;

        // while curr < pos && i < (*store_lock).total_store.list.len() {
        //     // TODO: what if pos < curr and the loop already break?
        //     if !(*store_lock).total_store.list[i].is_deleted {
        //         curr += (*store_lock).total_store.list[i].content.content.len() as u32;
        //     }
        //     i += 1;
        // }

        // if curr == pos {
        //     // Delete to i-th position in total_store
        //     (*store_lock).delete((*store_lock).total_store.list[i - 1].clone());
        // } else {
        //     // Have to split total_store[i-1]
        //     let left_id = Some((*store_lock).total_store.list[i - 1].id.clone());
        //     let left_content_len = (*store_lock).total_store.list.len();
        //     new_block.left_origin = left_id.clone();
        //     new_block.right_origin = Some(BlockID::new(
        //         left_id.clone().unwrap().client,
        //         left_id.clone().unwrap().clock + left_content_len as u32 - (curr - pos),
        //     ));
        //     (*store_lock).insert(new_block, left_id);
        // }

        // Update vector clock
        // self.vector_clock.increment(self.client);
    }

    pub async fn to_string(&self) -> String {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        (*store_lock).to_string()
    }
}
