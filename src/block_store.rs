use crate::block::Block;
use crate::block::BlockID;
use crate::list::Content;
use std::collections::HashMap;
pub struct BlockList {
    list: Vec<Block>,
}

impl BlockList {
    pub fn new() -> Self {
        BlockList { list: Vec::new() }
    }
}

pub struct BlockStore {
    kvStore: HashMap<BlockID, BlockList>,
    totalStore: BlockList,
}

impl BlockStore {
    pub fn new() -> Self {
        BlockStore {
            kvStore: HashMap::new(),
            totalStore: BlockList::new(),
        }
    }

    // Insert the content into pos in BlockStore
    pub fn insert(content: Content, pos: u32) {}

    // Delete the content of length len from pos
    pub fn delete(pos: u32, len: u32) {}

    // Split the block into a part of len
    // and rest of the block
    pub fn split(block: Block, len: u32) {}
}
