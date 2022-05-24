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

        let mut new_block_clk;
        if store_lock.kv_store.contains_key(&self.client) {
            let list = store_lock.kv_store.get(&self.client).unwrap().list.clone();
            if list.len() == 0 {
                new_block_clk = 0;
            } else {
                new_block_clk = list.last().unwrap().id.clock
                    + list.last().unwrap().content.content.len() as u32;
            }
        } else {
            new_block_clk = 0;
        }

        // Create a new block and insert it to total_store
        let mut new_block = Block {
            id: BlockID {
                client: self.client,
                clock: new_block_clk,
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
            let left_content_len =
                (*store_lock).total_store.list[i - 1].content.content.len() as u32;
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some(BlockID::new(
                left_id.clone().unwrap().client,
                left_id.clone().unwrap().clock + left_content_len - (curr - pos),
            ));

            // Split the block
            (*store_lock).split(left_id.clone().unwrap(), left_content_len - (curr - pos));
            (*store_lock).insert(new_block, left_id);
        }

        // Update vector clock
        self.vector_clock.increment(self.client);
    }

    // Delete the content of length len from pos
    pub async fn delete_remote(&self, update: Updates) {}
    pub async fn delete_local(&mut self, pos: u32, len: u32) {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;
        let mut delete_list: Vec<Block> = vec![];

        // Find the correct blocks to delete
        // The block may need to be splitted
        let mut i_start = 0 as usize;
        let mut curr_start = -1;

        loop {
            // TODO: what if pos < curr_left and the loop already break?
            if !(*store_lock).total_store.list[i_start].is_deleted {
                curr_start += (*store_lock).total_store.list[i_start]
                    .content
                    .content
                    .len() as i32;
            }
            if curr_start < pos as i32 {
                i_start += 1;
            } else {
                break;
            }
        }

        let mut i_end = i_start;
        let mut curr_end = curr_start;
        let mut pos_end = pos + len - 1;
        loop {
            if curr_end < pos_end as i32 {
                i_end += 1;
            } else {
                break;
            }
            if !(*store_lock).total_store.list[i_end].is_deleted {
                curr_end += (*store_lock).total_store.list[i_end].content.content.len() as i32;
            }
        }

        // Delete all blocks in (i_start, i_end) directly
        // Delete i_start and i_end according to the position
        if i_start == i_end {
            // All texts to be deleted are in the same block
            let block_id = (*store_lock).total_store.list[i_start].id.clone();
            let length = (*store_lock).total_store.list[i_start]
                .content
                .content
                .len() as u32;
            // split into three Blocks
            // the middle one will be deleted
            let left_length = length - (curr_start as u32 - pos) + 1;
            let mut new_blockID;
            if left_length != 0 {
                (*store_lock).split(block_id.clone(), left_length);
                new_blockID = BlockID::new(
                    block_id.client.clone(),
                    block_id.clock.clone() + left_length,
                );
            } else {
                new_blockID = block_id.clone();
            }

            if pos_end as i32 == curr_start {
                (*store_lock).delete(new_blockID);
            } else {
                let mid_length = pos_end - pos + 1;
                (*store_lock).split(new_blockID.clone(), mid_length);
                (*store_lock).delete(new_blockID);
            }
        } else {
            // Delete all blocks in between
            let mut i = i_start + 1;
            while i < i_end {
                let block_id = (*store_lock).total_store.list[i].id.clone();
                (*store_lock).delete(block_id);
            }

            // Delete left blocks
            let length_start = (*store_lock).total_store.list[i_start]
                .content
                .content
                .len() as u32;
            let block_id_start = (*store_lock).total_store.list[i_start].id.clone();
            let left_length = length_start - curr_start as u32 + pos - 1;
            let mut new_blockID;
            if left_length != 0 {
                (*store_lock).split(block_id_start.clone(), left_length);
                new_blockID = BlockID::new(
                    block_id_start.client.clone(),
                    block_id_start.clock.clone() + left_length,
                );
            } else {
                new_blockID = block_id_start.clone();
            }
            (*store_lock).delete(new_blockID);

            // Delete right blocks
            let length_end = (*store_lock).total_store.list[i_end].content.content.len() as u32;
            let block_id_end = (*store_lock).total_store.list[i_end].id.clone();
            let right_length = length_end - curr_end as u32 + pos;
            if curr_end != pos_end as i32 {
                (*store_lock).split(block_id_end.clone(), right_length);
            }
            (*store_lock).delete(block_id_end);
        }

        // Update vector clock
        self.vector_clock.increment(self.client);
    }

    pub async fn to_string(&self) -> String {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        (*store_lock).to_string()
    }
}
