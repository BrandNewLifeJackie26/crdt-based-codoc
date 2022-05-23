use std::sync::Arc;

pub struct Doc {
    name: String,
    block_store: Arc<Mutex<BlockStore>>,
}

impl Doc {}
