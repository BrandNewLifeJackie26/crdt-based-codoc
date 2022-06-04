use tokio::sync::Mutex;

use crate::block::{Block, BlockID, BlockPtr, Content};
use crate::utils::{ClientID, Updates};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type BlockListPtr = Box<BlockList>;

// BlockList is a list of blocks,
// the position in the vector indicates its spatial order
//
// i.e. if block represents text, ["1", "2"] represents string "12"
#[derive(Debug)]
pub struct BlockList {
    pub list: Vec<BlockPtr>,
}

impl BlockList {
    pub fn new() -> Self {
        BlockList { list: Vec::new() }
    }

    pub async fn getList(&self) -> Updates {
        let mut res = vec![];
        for block in &self.list {
            let t_b = block.lock().await;
            res.push(t_b.clone());
        }
        return res;
    }
}

// BlockStore is a collection of current blocks
// 1. kvStore stores a mapping from client to the changes the client made
// 2. totalStore stores the SPATIAL order of the blocks
//
// IMPORTANT: BlockStore is only a collections of data, it is stateless (states are in Doc)
// it also cannot be modified except by Doc
// TODO: block_map, kv_store and total_store are not using the same block
pub struct BlockStore {
    pub block_map: HashMap<BlockID, BlockPtr>,
    pub kv_store: HashMap<ClientID, BlockList>,
    pub total_store: BlockList,
}

impl BlockStore {
    pub fn new() -> Self {
        BlockStore {
            block_map: HashMap::new(),
            kv_store: HashMap::new(),
            total_store: BlockList::new(),
        }
    }

    pub async fn exist(&self, block: &Block) -> bool {
        for b in &self.total_store.list {
            let t_b = b.lock().await;
            if t_b.id.client == block.id.client && t_b.id.clock == block.id.clock {
                return true;
            }
        }
        return false;
    }

    // Insert the new block to the position
    // right to the block with BlockID left_id
    pub async fn insert(&mut self, block: Block, left_id: Option<BlockID>) {
        // println!(
        //     "Calling insert, block = {:?}, left_id = {:?},total_store = {:?}",
        //     block, left_id, self.total_store.list
        // );

        let block_id = block.id.clone();
        let block_ptr = Arc::new(Mutex::new(block));

        match left_id {
            Some(left_id) => {
                let mut i = 0 as usize;
                for b in &self.total_store.list {
                    {
                        let b_lock = b.lock().await;
                        if b_lock.id == left_id {
                            break;
                        }
                    }
                    i += 1;
                }
                self.total_store.list.insert(i + 1, block_ptr.clone());
            }
            None => {
                self.total_store.list.insert(0, block_ptr.clone());
            }
        }

        // Update BlockStore state
        self.update_state(block_id, block_ptr.clone());
    }

    // Delete the content of length len from pos
    pub async fn delete(&mut self, block_id: BlockID) {
        let block = self.block_map.get_mut(&block_id);
        if let Some(block) = block {
            let mut block_lock = block.lock().await;
            block_lock.delete();
        }
    }

    // Update block content
    pub async fn update(&mut self, new_block: &Block) {
        let block = self.block_map.get_mut(&new_block.id);
        if let Some(block) = block {
            let mut block_lock = block.lock().await;
            block_lock.content.content = new_block.content.content.clone();
        }
    }

    // optimization: Split the block into a part of len
    // and rest of the block
    pub async fn split(&mut self, block_id: BlockID, len: u32) {
        let block = {
            if let Some(block_ptr) = self.block_map.get(&block_id) {
                Some(block_ptr)
            } else {
                None
            }
        };
        // println!(
        //     "Calling split, blockid = {:?}, block = {:?}, len = {}",
        //     block_id.clone(),
        //     block,
        //     len
        // );

        let len = len as usize;
        let mut right_block: Option<Block> = None;
        if let Some(block) = block {
            let mut block_lock = block.lock().await;

            // It is impossible to split the block into a part
            // that has a longer content than the original
            if len > block_lock.content.content.len() {
                return;
            }

            let left_content = Content {
                content: String::from(&block_lock.content.content[..len]),
            };
            let right_content = Content {
                content: String::from(&block_lock.content.content[len..]),
            };

            // Create a new block to hold right content
            let right_block_id = BlockID {
                client: block_lock.id.client,
                clock: block_lock.id.clock + len as u32,
            };
            right_block = Some(Block {
                id: right_block_id.clone(),
                left_origin: Some(block_id.clone()),
                right_origin: block_lock.right_origin.clone(),
                is_deleted: false,
                content: right_content,
            });

            // Modify the left block
            block_lock.content = left_content;
            block_lock.right_origin = Some(right_block_id.clone());
        }

        if let Some(right_block) = right_block {
            self.insert(right_block, Some(block_id.clone())).await;
        }
    }

    // optimization: Squash list of blocks into one
    pub async fn squash(&mut self, block_ids: Vec<BlockID>) {}

    // Form a string by connecting all elements in the current BlockList
    pub async fn to_string(&self) -> String {
        // println!(
        //     "Calling to_string, total_store: {:?}",
        //     self.total_store.list
        // );

        let mut res: Vec<String> = vec![];
        for block in &self.total_store.list {
            let block_lock = block.lock().await;
            if block_lock.is_deleted {
                continue;
            }
            res.push(block_lock.content.content.clone());
        }
        res.into_iter().collect()
    }

    // Update BlockStore state
    fn update_state(&mut self, new_block_id: BlockID, new_block: Arc<Mutex<Block>>) {
        self.block_map
            .insert(new_block_id.clone(), new_block.clone());

        let client_block_list = self.kv_store.get_mut(&new_block_id.client);
        match client_block_list {
            None => {
                self.kv_store.insert(
                    new_block_id.client,
                    BlockList {
                        list: vec![new_block.clone()],
                    },
                );
            }
            Some(client_block_list) => {
                client_block_list.list.push(new_block.clone());
            }
        };
    }
}
