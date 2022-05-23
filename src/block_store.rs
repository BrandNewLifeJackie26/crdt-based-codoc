use crate::block::Block;
use crate::block::BlockID;
use std::collections::HashMap;

pub type ClientID = u32;

pub struct BlockList {
    list: Vec<Block>,
}

pub struct VectorClock {
    clockMap: HashMap<ClientID, u32>,
}

impl VectorClock {
    pub fn from () -> VectorClock {todo!()}
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
