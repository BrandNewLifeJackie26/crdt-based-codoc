use crate::block::Block;
use crate::block::BlockID;
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

    pub fn insert(new_block: Block, pos: u32) {}
}
