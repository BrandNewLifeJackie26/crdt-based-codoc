use crate::utils::ClientID;
use serde::Serialize;
#[derive(Clone, Serialize)]
pub struct Content {
    pub content: String,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct BlockID {
    pub client: ClientID,
    pub clock: u32,
}

impl BlockID {
    pub fn new(client: ClientID, clock: u32) -> Self {
        BlockID { client, clock }
    }
}

// Block is the basic building block of doc (e.g. text, xml element, etc.),
// one block can be split to two blocks,
// and two blocks can be merged into one
#[derive(Clone, Serialize)]
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
        self.is_deleted = false;
    }
}
