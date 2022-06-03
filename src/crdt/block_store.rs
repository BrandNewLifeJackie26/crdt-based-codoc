use tokio::sync::Mutex;

use crate::crdt::block::{Block, BlockID, BlockPtr, Content};
use crate::crdt::utils::ClientID;
use std::collections::HashMap;
use std::sync::Arc;

pub type BlockListPtr = Box<BlockList>;

// BlockList is a list of blocks,
// the position in the vector indicates its spatial order
//
// i.e. if block represents text, ["1", "2"] represents string "12"
#[derive(Debug)]
pub struct BlockList {
    pub list: Vec<BlockPtr>,
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
// TODO: block_map, kv_store and total_store are not using the same block
pub struct BlockStore {
    pub block_map: HashMap<BlockID, BlockPtr>,
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

    // Insert the new block to the position
    // right to the block with BlockID left_id
    pub async fn insert(&mut self, block: Block, left_id: Option<BlockID>) {
        // println!(
        //     "Calling insert, block = {:?}, left_id = {:?},total_store = {:?}",
        //     block, left_id, self.total_store.list
        // );

        let block_id = block.id.clone();
        let block_ptr = Arc::new(Mutex::new(block));

        match left_id {
            Some(left_id) => {
                let mut i = 0 as usize;
                for b in &self.total_store.list {
                    {
                        let b_lock = b.lock().await;
                        if b_lock.id == left_id {
                            break;
                        }
                    }
                    i += 1;
                }
                self.total_store.list.insert(i + 1, block_ptr.clone());
            }
            None => {
                self.total_store.list.insert(0, block_ptr.clone());
            }
        }

        // Update BlockStore state
        self.update_state(block_id, block_ptr.clone());
    }

    // Delete the content of length len from pos
    pub async fn delete(&mut self, block_id: BlockID) {
        let block = self.block_map.get_mut(&block_id);
        if let Some(block) = block {
            let mut block_lock = block.lock().await;
            block_lock.delete();
        }
    }

    // optimization: Split the block into a part of len
    // and rest of the block
    pub async fn split(&mut self, block_id: BlockID, len: u32) {
        let block = {
            if let Some(block_ptr) = self.block_map.get(&block_id) {
                Some(block_ptr)
            } else {
                None
            }
        };
        // println!(
        //     "Calling split, blockid = {:?}, block = {:?}, len = {}",
        //     block_id.clone(),
        //     block,
        //     len
        // );

        let len = len as usize;
        let mut right_block: Option<Block> = None;
        if let Some(block) = block {
            let mut block_lock = block.lock().await;

            // It is impossible to split the block into a part
            // that has a longer content than the original
            if len > block_lock.content.content.len() {
                return;
            }

            let left_content = Content {
                content: String::from(&block_lock.content.content[..len]),
            };
            let right_content = Content {
                content: String::from(&block_lock.content.content[len..]),
            };

            // Create a new block to hold right content
            let right_block_id = BlockID {
                client: block_lock.id.client,
                clock: block_lock.id.clock + len as u32,
            };
            right_block = Some(Block {
                id: right_block_id.clone(),
                left_origin: Some(block_id.clone()),
                right_origin: block_lock.right_origin.clone(),
                is_deleted: false,
                content: right_content,
            });

            // Modify the left block
            block_lock.content = left_content;
            block_lock.right_origin = Some(right_block_id.clone());
        }

        if let Some(right_block) = right_block {
            self.insert(right_block, Some(block_id.clone())).await;
        }
    }

    // optimization: Squash valid neighboring blocks
    // IMPORTANT: Only those that have just been inserted (after the latest update to another client)
    // can be squashed, so that we can avoid cases such as:
    // 1. Delete more blocks while the original block is not intended to be squashed
    //
    // Some expections:
    // 1. delay/time cost may deteriorate a lot if frequently insert into large chunk of data
    // 2. memory efficiency will be improved
    // 3. This performs better than yrs in that we can merge split blocks

    // TODO:
    // 1. can be commented out for perf evaluation
    // 2. interleaving? <content{id, left_origin, right_origin}>
    //    A: a{(A,0), None, None} => a{(A,0), None, (A,1)}b{(A,1), (A,0), None} =(squash)=> ab{(A,0), None, None}
    // (cannot merge because it is sent to another client)
    //    B: a{(A,0), None, None} => a{(A,0), None, (B,0)}c{(B,0), (A,0), None}
    // (how to merge? if consistent, no problem)
    pub async fn squash(&mut self, block_id: BlockID, latest_clock: Option<u32>) {
        // 3 possible scenarios:
        // 1. insert after
        // 2. insert before
        // 3. split and insert

        let no_sync = latest_clock.is_none(); // No sync so far, all blocks can be squashed

        let mut middle = 0 as usize;
        while middle < self.total_store.list.len() {
            if self.total_store.list[middle].lock().await.id == block_id {
                break;
            }
            middle += 1;
        }

        let (middle_id, mut middle_content, middle_right_origin) = {
            let middle_lock = self.total_store.list[middle].lock().await;
            (
                block_id,
                middle_lock.content.content.clone(),
                middle_lock.right_origin.clone(),
            )
        };
        let left_block = {
            if middle == 0 {
                None
            } else {
                self.total_store.list.get(middle - 1)
            }
        };
        let right_block = {
            if middle >= self.total_store.list.len() - 1 {
                None
            } else {
                self.total_store.list.get(middle + 1)
            }
        };

        if let Some(left_block) = left_block {
            let (left_id, mut left_content, left_left_origin, left_deleted) = {
                let left_lock = left_block.lock().await;
                (
                    left_lock.id.clone(),
                    left_lock.content.content.clone(),
                    left_lock.left_origin.clone(),
                    left_lock.is_deleted,
                )
            };

            // Scenario1: insert after (squash left and current)
            if left_id.client == middle_id.client
                && !left_deleted
                && (no_sync || left_id.clock > latest_clock.unwrap())
                && left_id.clock + left_content.len() as u32 == middle_id.clock
            {
                // Merge two blocks into the left block
                {
                    let mut left_lock = left_block.lock().await;
                    left_content.push_str(&middle_content);
                    left_lock.content = Content {
                        content: left_content,
                    };
                    left_lock.right_origin = middle_right_origin;
                }

                // Remove the middle block from states
                self.remove_state(middle_id).await;
                return;
            };

            // Scenario3: split and insert (squash left, current and right)
            if let Some(right_block) = right_block {
                let (right_id, right_content, right_right_origin, right_deleted) = {
                    let right_lock = right_block.lock().await;
                    (
                        right_lock.id.clone(),
                        right_lock.content.content.clone(),
                        right_lock.right_origin.clone(),
                        right_lock.is_deleted,
                    )
                };

                if left_id.client == middle_id.client
                    && middle_id.client == right_id.client
                    && !left_deleted
                    && !right_deleted
                    && (no_sync
                        || (left_id.clock > latest_clock.unwrap()
                            && right_id.clock > latest_clock.unwrap()))
                    && (left_id.clock + left_content.len() as u32 == right_id.clock
                        && right_id.clock + right_content.len() as u32 == middle_id.clock)
                {
                    // Merge three blocks into the left block
                    {
                        let mut left_lock = left_block.lock().await;
                        left_content.push_str(&middle_content);
                        left_content.push_str(&right_content);
                        left_lock.content = Content {
                            content: left_content,
                        };
                        left_lock.left_origin = left_left_origin;
                        left_lock.right_origin = right_right_origin;
                    }

                    // Remove the middle and right blocks from states
                    self.remove_state(middle_id).await;
                    self.remove_state(right_id).await;
                    return;
                }
            }

            return;
        }

        // Scenario2: insert before (squash current and right)
        if let Some(right_block) = right_block {
            let (right_id, right_content, right_right_origin, right_deleted) = {
                let right_lock = right_block.lock().await;
                (
                    right_lock.id.clone(),
                    right_lock.content.content.clone(),
                    right_lock.right_origin.clone(),
                    right_lock.is_deleted,
                )
            };

            // Merge two blocks into the middle block
            {
                let mut middle_lock = self.total_store.list[middle].lock().await;
                middle_content.push_str(&right_content);
                middle_lock.content = Content {
                    content: middle_content,
                };
                middle_lock.right_origin = right_right_origin;
            }

            // Remove the right block from states
            self.remove_state(right_id).await;
        }
    }

    // Form a string by connecting all elements in the current BlockList
    pub async fn to_string(&self) -> String {
        // println!(
        //     "Calling to_string, total_store: {:?}",
        //     self.total_store.list
        // );

        let mut res: Vec<String> = vec![];
        for block in &self.total_store.list {
            let block_lock = block.lock().await;
            if block_lock.is_deleted {
                continue;
            }
            res.push(block_lock.content.content.clone());
        }
        res.into_iter().collect()
    }

    // Update BlockStore state
    fn update_state(&mut self, new_block_id: BlockID, new_block: Arc<Mutex<Block>>) {
        self.block_map
            .insert(new_block_id.clone(), new_block.clone());

        let client_block_list = self.kv_store.get_mut(&new_block_id.client);
        match client_block_list {
            None => {
                self.kv_store.insert(
                    new_block_id.client,
                    BlockList {
                        list: vec![new_block.clone()],
                    },
                );
            }
            Some(client_block_list) => {
                client_block_list.list.push(new_block.clone());
            }
        };
    }

    // Remove state of the block from all BlockStore states
    async fn remove_state(&mut self, block_id: BlockID) {
        let mut i = 0;
        while i < self.total_store.list.len() {
            if self.total_store.list[i].lock().await.id == block_id {
                break;
            }
            i += 1;
        }
        self.total_store.list.remove(i);

        self.block_map.remove(&block_id);

        i = 0;
        while i < self.kv_store.get(&block_id.client).unwrap().list.len() {
            if self.kv_store.get(&block_id.client).unwrap().list[i]
                .lock()
                .await
                .id
                == block_id
            {
                break;
            }
            i += 1;
        }
        self.kv_store
            .get_mut(&block_id.client)
            .unwrap()
            .list
            .remove(i);
    }
}
