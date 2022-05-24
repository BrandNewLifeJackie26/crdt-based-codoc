use crate::block::{Block, ClientID, BlockID, Content};
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

    pub fn get_current_clock(&self) -> u32 {
        let mut clock = 0;
        for block in self.total_store.list.iter() {
            clock += block.content.content.len();
        }
        clock as u32
    }

    pub fn insert(&mut self, block: Block, left_id: Option<BlockID>) {
        match left_id {
            Some(left_id) => {
                let mut i = 0 as usize;
                for b in self.total_store.list.iter() {
                    if b.id == left_id {
                        break;
                    }
                    i += 1;
                }
                self.total_store.list.insert(i+1, block);
            },
            None => {
                self.total_store.list.insert(0, block);
            }
        }
    }

    // Delete the content of length len from pos
    pub fn delete(&self, client: ClientID, pos: u32, len: u32) {}

    // optimization: Split the block into a part of len
    // and rest of the block
    pub fn split(&self, client: ClientID, block: Block, len: u32) {}

    // optimization: Squash list of blocks into one
    pub fn squash(&self, client: ClientID, block_list: Vec<Block>) {}

    // Form a string by connecting all elements in the current BlockList
    pub fn to_string(&self) -> String {
        let mut res: Vec<String> = vec![];
        for block in self.total_store.list.iter() {
            res.push(block.content.content.clone());
        }
        res.into_iter().collect()
    }
}
