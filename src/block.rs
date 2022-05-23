pub type ClientID = u32;

#[derive(Clone)]
pub struct Content {
    pub content: String,
}

pub struct BlockID {
    client: ClientID,
    clock: u32,
}

impl BlockID {
    pub fn new(client: ClientID, clock: u32) -> Self {
        BlockID { client, clock }
    }
}

// Block is the basic building block of doc (e.g. text, xml element, etc.),
// one block can be split to two blocks,
// and two blocks can be merged into one
pub struct Block {
    id: BlockID,
    left_origin: BlockID,
    right_origin: BlockID,
    is_deleted: bool,
    content: Content,
}

impl Block {
    pub fn new(id: BlockID, left_origin: BlockID, right_origin: BlockID, content: Content) -> Self {
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
