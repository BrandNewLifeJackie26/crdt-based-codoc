use crate::block::{Block, BlockID};
use crate::utils::ClientID;
use std::collections::HashMap;

// BlockList is a list of blocks,
// the position in the vector indicates its spatial order
//
// i.e. if block represents text, ["1", "2"] represents string "12"
pub struct BlockList {
    pub list: Vec<Block>,
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
pub struct BlockStore {
    pub block_map: HashMap<BlockID, Block>,
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

    pub fn insert(&mut self, block: Block, left_id: Option<BlockID>) {
        let block_id = block.clone().id;
        match left_id {
            Some(left_id) => {
                let mut i = 0 as usize;
                for b in self.total_store.list.iter() {
                    if b.id == left_id {
                        break;
                    }
                    i += 1;
                }
                self.total_store.list.insert(i + 1, block.clone());
            }
            None => {
                self.total_store.list.insert(0, block.clone());
            }
        }

        // Update BlockStore state
        self.block_map.insert(block_id.clone(), block.clone());
        let client_block_list = self.kv_store.get_mut(&block_id.client);
        match client_block_list {
            None => {
                self.kv_store.insert(
                    block_id.client,
                    BlockList {
                        list: vec![block.clone()],
                    },
                );
            }
            Some(client_block_list) => {
                client_block_list.list.push(block.clone());
            }
        };
    }

    // Delete the content of length len from pos
    pub fn delete(&self, block: Block) {}

    // optimization: Split the block into a part of len
    // and rest of the block
    pub fn split(&self, block_id: BlockID, len: u32) {
        // let block = self.block_map.get(&block_id);
        // if let Some(block) = block {
        //     let old_content = String::from(&block.content.content[..len as usize]);
        //     let new_content = String::from(&block.content.content[len as usize..]);
        // }
    }

    // optimization: Squash list of blocks into one
    pub fn squash(&self, block_ids: Vec<BlockID>) {}

    // Form a string by connecting all elements in the current BlockList
    pub fn to_string(&self) -> String {
        let mut res: Vec<String> = vec![];
        for block in self.total_store.list.iter() {
            res.push(block.content.content.clone());
        }
        res.into_iter().collect()
    }
}
