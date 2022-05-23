use crate::block::{Block, ClientID, BlockID, Content};
use std::collections::HashMap;

// BlockList is a list of blocks,
// the position in the vector indicates its spatial order
//
// i.e. if block represents text, ["1", "2"] represents string "12"
pub struct BlockList {
    list: Vec<Block>,
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
    kv_store: HashMap<ClientID, BlockList>,
    total_store: BlockList,
}

impl BlockStore {
    pub fn new() -> Self {
        BlockStore {
            kv_store: HashMap::new(),
            total_store: BlockList::new(),
        }
    }

    // Insert the content into pos in BlockStore
    pub fn insert(&self, content: Content, pos: u32) {}

    // Delete the content of length len from pos
    pub fn delete(&self, pos: u32, len: u32) {}

    // optimization: Split the block into a part of len
    // and rest of the block
    pub fn split(&self, block: Block, len: u32) {}

    // optimization: Squash list of blocks into one
    pub fn squash(&self, block_list: Vec<Block>) {}
}
