use crate::block::{Block, BlockID, BlockPtr, Content};
use crate::utils::ClientID;
use std::collections::HashMap;

pub type BlockListPtr = Box<BlockList>;

// BlockList is a list of blocks,
// the position in the vector indicates its spatial order
//
// i.e. if block represents text, ["1", "2"] represents string "12"
pub struct BlockList {
    pub list: Vec<BlockPtr>,
}

impl BlockList {
    pub fn new() -> Self {
        BlockList { list: Vec::new() }
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
    pub block_map: Box<HashMap<BlockID, BlockPtr>>,
    pub kv_store: Box<HashMap<ClientID, BlockListPtr>>,
    pub total_store: BlockListPtr,
}

impl BlockStore {
    pub fn new() -> Self {
        BlockStore {
            block_map: Box::new(HashMap::new()),
            kv_store: Box::new(HashMap::new()),
            total_store: Box::new(BlockList::new()),
        }
    }

    // Insert the new block to the position
    // right to the block with BlockID left_id
    pub fn insert(&mut self, block: Block, left_id: Option<BlockID>) {
        let block_id = block.id.clone();
        let block_ptr = Box::new(block);

        let total_store = self.total_store.as_mut();
        match left_id {
            Some(left_id) => {
                let mut i = 0 as usize;
                for b in total_store.list.iter() {
                    if b.id == left_id {
                        break;
                    }
                    i += 1;
                }
                total_store.list.insert(i + 1, block_ptr.clone());
            }
            None => {
                total_store.list.insert(0, block_ptr.clone());
            }
        }

        // Update BlockStore state
        self.update_state(block_id, block_ptr.clone());
    }

    // Delete the content of length len from pos
    pub fn delete(&self, block_id: BlockID) {
        let mut block_map = self.block_map.clone();
        let block = block_map.get_mut(&block_id);
        if let Some(block) = block {
            block.delete();
        }
    }

    // optimization: Split the block into a part of len
    // and rest of the block
    pub fn split(&mut self, block_id: BlockID, len: u32) {
        let block = {
            let block_map = self.block_map.as_mut();

            if let Some(block_ptr) = block_map.get_mut(&block_id) {
                Some(block_ptr)
            } else {
                None
            }
        };
        println!(
            "Calling split, blockid = {:?}, block = {:?}, len = {}",
            block_id.clone(),
            block,
            len
        );

        let len = len as usize;
        if let Some(block) = block {
            // It is impossible to split the block into a part
            // that has a longer content than the original
            if len > block.content.content.len() {
                return;
            }

            let left_content = Content {
                content: String::from(&block.content.content[..len]),
            };
            let right_content = Content {
                content: String::from(&block.content.content[len..]),
            };

            // Create a new block to hold right content
            let right_block_id = BlockID {
                client: block.id.client,
                clock: block.id.clock + len as u32,
            };
            let right_block = Block {
                id: right_block_id.clone(),
                left_origin: Some(block_id.clone()),
                right_origin: block.right_origin.clone(),
                is_deleted: false,
                content: right_content,
            };

            // Modify the left block
            block.content = left_content;
            block.right_origin = Some(right_block_id.clone());
            self.insert(right_block, Some(block_id.clone()));
        }
    }

    // optimization: Squash list of blocks into one
    pub async fn squash(&mut self, block_ids: Vec<BlockID>) {}

    // Form a string by connecting all elements in the current BlockList
    pub fn to_string(&self) -> String {
        let mut res: Vec<String> = vec![];
        for block in self.total_store.list.iter() {
            res.push(block.content.content.clone());
        }
        res.into_iter().collect()
    }

    // Update BlockStore state
    fn update_state(&mut self, new_block_id: BlockID, new_block: Box<Block>) {
        let block_map = self.block_map.as_mut();
        let kv_store = self.kv_store.as_mut();

        block_map.insert(new_block_id.clone(), new_block.clone());
        let client_block_list = kv_store.get_mut(&new_block_id.client);
        match client_block_list {
            None => {
                kv_store.insert(
                    new_block_id.client,
                    Box::new(BlockList {
                        list: vec![new_block],
                    }),
                );
            }
            Some(client_block_list) => {
                client_block_list.list.push(new_block);
            }
        };
    }
}
