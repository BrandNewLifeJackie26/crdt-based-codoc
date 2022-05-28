use crate::utils::ClientID;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Content {
    pub content: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
pub struct BlockID {
    pub client: ClientID,
    pub clock: u32,
}

impl Ord for BlockID {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.client == other.client {
            self.clock.cmp(&other.clock)
        } else {
            self.client.cmp(&other.client)
        }
    }
}

impl PartialOrd for BlockID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.client == other.client {
            Some(self.clock.cmp(&other.clock))
        } else {
            Some(self.client.cmp(&other.client))
        }
    }
}

impl BlockID {
    pub fn new(client: ClientID, clock: u32) -> Self {
        BlockID { client, clock }
    }
}

pub type BlockPtr = Box<Block>;

// Block is the basic building block of doc (e.g. text, xml element, etc.),
// one block can be split to two blocks,
// and two blocks can be merged into one
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub id: BlockID,
    pub left_origin: Option<BlockID>,
    pub right_origin: Option<BlockID>,
    pub is_deleted: bool,
    pub content: Content,
}

impl Block {
    pub fn new(
        id: BlockID,
        left_origin: Option<BlockID>,
        right_origin: Option<BlockID>,
        content: Content,
    ) -> Self {
        Block {
            id,
            left_origin,
            right_origin,
            is_deleted: false,
            content,
        }
    }

    // Delete the current block (mark as deleted)
    pub fn delete(&mut self) {
        self.is_deleted = true;
    }
}
