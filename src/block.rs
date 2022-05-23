pub type ClientID = u32;

pub struct BlockID {
    client: ClientID,
    clock: u32,
}

impl BlockID {
    pub fn new(client: ClientID, clock: u32) -> Self {
        BlockID { client, clock }
    }
}

pub struct Block {
    id: BlockID,
    left_origin: BlockID,
    right_origin: BlockID,
    is_deleted: bool,
    content: String,
}

impl Block {
    pub fn new(id: BlockID, left_origin: BlockID, right_origin: BlockID, content: String) -> Self {
        Block {
            id,
            left_origin,
            right_origin,
            is_deleted: false,
            content,
        }
    }

    pub fn delete(&mut self) {
        self.is_deleted = false;
    }
}
