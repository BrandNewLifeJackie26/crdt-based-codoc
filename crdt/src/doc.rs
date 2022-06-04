use crate::utils::{ClientID, Peer, Updates};
use crate::{block::Block, block::BlockID, block::Content, block_store::BlockStore};
use std::cmp::{max, min};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

// VectorClock represents the latest clocks of all clients,
// it is used during synchronization to find the missing changes

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VectorClock {
    pub clock_map: HashMap<ClientID, u32>,
}

impl VectorClock {
    pub fn new() -> VectorClock {
        VectorClock {
            clock_map: HashMap::new(),
        }
    }

    pub fn from() -> VectorClock {
        todo!()
    }

    pub fn increment(&mut self, client: ClientID, val: usize) {
        self.clock_map.insert(
            client,
            self.clock_map.get(&client).cloned().unwrap_or(0) + val as u32,
        );
    }
}

// Doc is the collaborative edited document,
// it is owned by client, block_store is the real storage of all elements inside the doc.
//
// Doc also stores some pending updates to avoid out-of-order updates
//
// IMPORTANT: Doc = block_store + state
#[derive(Clone)]
pub struct Doc {
    pub name: String,
    pub client: ClientID,
    pub block_store: Arc<Mutex<BlockStore>>,
    // list of peers that are collaborately editing the same doc
    pub peers: Vec<Peer>,
    pub pending_updates: Updates,
    // TODO: states: vector clock, pending updates, delete set, etc.
    pub vector_clock: VectorClock,
}

impl Doc {
    pub fn new(name: String, client: ClientID) -> Self {
        Doc {
            name,
            client,
            block_store: Arc::new(Mutex::new(BlockStore::new())),
            peers: vec![],
            pending_updates: vec![],
            vector_clock: VectorClock {
                clock_map: HashMap::new(),
            },
        }
    }

    /* Local operations */
    // TODO: local operations should also grab mutex of the whole doc (as in SyncTransaction) to avoid concurrency issue
    pub async fn insert_remote(&mut self, update: Updates, peer_id: u32) {
        for block in update.iter() {
            // Try insert pending updates
            self.flush_pending_updates().await; // TODO: flush every time an insersion happens? Is it possible that current insersion and remote update interleave?
                                                // Try insert current updates
            {
                // If the block already exists, update the block content
                let t_b = self.block_store.lock().await;
                if t_b.exist(block).await {
                    continue;
                }
            }
            let success = self.insert_single_block(block).await;
            self.vector_clock
                .increment(peer_id, block.content.content.len());
            if !success {
                self.pending_updates.push(block.clone());
            } else {
                self.flush_pending_updates().await; // TODO: flush every time an insersion happens? Is it possible that current insersion and remote update interleave?
            }
        }
    }

    pub async fn insert_single_block(&mut self, block: &Block) -> bool {
        // println!("insert single block");
        // Try insert, return false if failed, return true if success
        // First find the block corresponding the left_origin and right_origin
        // check if the block already exists

        {
            // If the block already exists, update the block content
            let t_b = self.block_store.lock().await;
            if t_b.exist(block).await {
                // t_b.update(block).await;
                return true;
            }
        }

        let left_res = self
            .find_block_idx(block.left_origin.clone(), 0, true)
            .await;
        if let Err(_) = left_res {
            println!("----!!!!Cannot find left block-----");
            return false;
        }

        let left = left_res.unwrap();
        let right_res = self
            .find_block_idx(block.right_origin.clone(), max(left, 0), false)
            .await;
        if let Err(_) = right_res {
            // not exist
            println!("----!!!!Cannot find right block-----");
            return false;
        }

        let right = right_res.unwrap();
        let mut i = (left + 1) as usize;
        let mut scan = false;
        let mut dest = (left + 1) as usize; // TODO: right?

        loop {
            let store = self.block_store.clone();
            let mut store_lock = store.lock().await;
            if !scan {
                dest = i;
            }
            if i == store_lock.total_store.list.len() || i == (right as usize) {
                break;
            }

            let curr = &store_lock.total_store.list[i];
            let (curr_id, curr_left_origin, curr_right_origin) = {
                let curr_lock = curr.lock().await;
                (
                    curr_lock.id.clone(),
                    curr_lock.left_origin.clone(),
                    curr_lock.right_origin.clone(),
                )
            };
            drop(store_lock);

            let curr_ol = self
                .find_block_idx(curr_left_origin, 0, true)
                .await
                .unwrap();
            let curr_or = self
                .find_block_idx(curr_right_origin, curr_ol, false)
                .await
                .unwrap();

            if curr_ol < left {
                break;
            } else if curr_ol == left {
                if curr_or < right {
                    scan = true;
                    i += 1;
                    continue;
                } else if curr_or == right {
                    if block.id < curr_id {
                        break;
                    } else {
                        scan = false;
                        i += 1;
                        continue;
                    }
                } else {
                    scan = false;
                    i += 1;
                    continue;
                }
            } else {
                i += 1;
                continue;
            }
        }

        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;
        let new_block = block.clone();
        let left_id;
        if dest == 0 {
            left_id = None;
        } else {
            let dest_lock = store_lock.total_store.list[dest - 1].lock().await;
            left_id = Some(dest_lock.id.clone());
        }
        store_lock.insert(new_block, left_id).await;
        true
    }

    pub async fn delete_single_block(&mut self, block: &Block) -> bool {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        let mut i = 0;
        let id = block.id.clone();
        while i < store_lock.total_store.list.len() {
            let (curr_id, curr_content) = {
                let curr_lock = store_lock.total_store.list[i].lock().await;
                (curr_lock.id.clone(), curr_lock.content.clone())
            };

            if curr_id == id {
                store_lock.delete(block.id.clone()).await;
                return true;
            } else if curr_id.client == id.client
                && curr_id.clock < id.clock
                && curr_id.clock + curr_content.content.len() as u32 - 1 >= id.clock
            {
                let block_to_split = curr_id.clone();
                let len = id.clock - curr_id.clock;
                store_lock.split(block_to_split, len).await;
                store_lock.delete(block.id.clone()).await;
                return true;
            }
            i += 1;
        }

        return false;
    }

    async fn find_block_idx(
        &mut self,
        block_id: Option<BlockID>,
        start_idx: i64,
        is_left: bool,
    ) -> Result<i64, bool> {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;
        match block_id {
            Some(id) => {
                let mut i: usize = start_idx as usize;
                while i < store_lock.total_store.list.len() {
                    let (curr_id, curr_content) = {
                        let curr_lock = store_lock.total_store.list[i].lock().await;
                        (curr_lock.id.clone(), curr_lock.content.clone())
                    };

                    if curr_id == id {
                        return Ok(i as i64);
                    } else if curr_id.client == id.client
                        && curr_id.clock < id.clock
                        && curr_id.clock + curr_content.content.len() as u32 - 1 >= id.clock
                    {
                        let block_to_split = curr_id.clone();
                        let len = id.clock - curr_id.clock;
                        store_lock.split(block_to_split, len).await;
                        return Ok((i + 1) as i64);
                    }
                    i += 1;
                }
                return Err(false);
            }
            None => {
                if is_left {
                    Ok(-1)
                } else {
                    Ok(store_lock.total_store.list.len().try_into().unwrap())
                }
            }
        }
    }

    // Insert the content into pos in BlockStore
    // TODO: Arc<Mutex<BlockList>>
    pub async fn insert_local(&mut self, content: Content, pos: u32) {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        // Find the correct block to insert
        let mut idx = 0 as usize;
        let mut prev_char_cnt = 0;

        while prev_char_cnt < pos && idx < store_lock.total_store.list.len() {
            // TODO: what if pos < curr and the loop already break?
            let (curr_is_deleted, curr_content) = {
                let curr_lock = store_lock.total_store.list[idx].lock().await;
                (curr_lock.is_deleted, curr_lock.content.clone())
            };

            if !curr_is_deleted {
                prev_char_cnt += curr_content.content.len() as u32;
            }
            idx += 1;
        }

        let new_block_clk;
        if store_lock.kv_store.contains_key(&self.client) {
            let list = &store_lock.kv_store.get(&self.client).unwrap().list;
            if list.len() == 0 {
                new_block_clk = 0;
            } else {
                let last_lock = list.last().unwrap().lock().await;
                new_block_clk = last_lock.id.clock + last_lock.content.content.len() as u32;
            }
        } else {
            new_block_clk = 0;
        }

        // Create a new block and insert it to total_store
        let mut new_block = Block {
            id: BlockID {
                client: self.client,
                clock: new_block_clk,
            },
            left_origin: None,
            right_origin: None,
            is_deleted: false,
            content: content.clone(),
        };

        // TODO:
        let (left_id, left_content) = {
            if idx > 0 {
                let left_lock = store_lock.total_store.list[idx - 1].lock().await;
                (Some(left_lock.id.clone()), left_lock.content.clone())
            } else {
                (None, Content::default()) // INVALID
            }
        };
        let curr_id = {
            if idx < store_lock.total_store.list.len() && prev_char_cnt == pos {
                let curr_lock = store_lock.total_store.list[idx].lock().await;
                curr_lock.id.clone()
            } else {
                BlockID::default() // INVALID
            }
        };

        if idx == store_lock.total_store.list.len()
            && pos >= store_lock.to_string().await.len() as u32
        {
            // Append to the end
            if idx > 0 {
                new_block.left_origin = left_id.clone();
                store_lock.insert(new_block, left_id).await;
            } else {
                store_lock.insert(new_block, None).await;
            }
        } else if prev_char_cnt == pos {
            // Insert to i-th position in total_store
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some(curr_id);
            store_lock.insert(new_block, left_id).await;
        } else {
            // Have to split total_store[i-1]
            let left_content_len = left_content.content.len() as u32;
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some(BlockID::new(
                left_id.clone().unwrap().client,
                left_id.clone().unwrap().clock + left_content_len - (prev_char_cnt - pos),
            ));

            // Split the block
            store_lock
                .split(
                    left_id.clone().unwrap(),
                    left_content_len - (prev_char_cnt - pos),
                )
                .await;
            store_lock.insert(new_block, left_id).await;
        }

        // Update vector clock
        self.vector_clock
            .increment(self.client, content.content.len());
    }

    // Delete the content of length len from pos
    pub async fn delete_remote(&mut self, update: Updates, peer_id: u32) {
        for block in update.iter() {
            // Try insert pending updates
            self.flush_pending_updates().await;
            // Try insert current updates
            let success = self.delete_single_block(block).await;
            if !success {
                println!("delete failed");
                self.pending_updates.push(block.clone());
            }
        }
    }

    async fn flush_pending_updates(&mut self) {
        let mut new_pending = vec![];
        for pending in self.pending_updates.clone().iter() {
            let success;
            if pending.is_deleted {
                success = self.delete_single_block(pending).await;
            } else {
                success = self.insert_single_block(pending).await;
            }
            if !success {
                new_pending.push(pending.clone());
            } else {
                println!(
                    "flush pending BLock: {:?}, content: {:?}",
                    pending.id, pending.content.content
                );
            }
        }
        self.pending_updates = new_pending;
        println!("Remaining: {:?}", self.pending_updates);
    }

    pub async fn delete_local(&mut self, pos: u32, len: u32) {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        // Pos out of range, no effect
        let doc_len = store_lock.to_string().await.len() as u32;
        if pos >= doc_len {
            return;
        }

        // Find the correct blocks to delete
        // The block may need to be splitted
        // TODO: empty doc -> raise error
        let mut left_idx = 0 as usize;
        let mut pos_limit_left = -1;

        loop {
            if left_idx >= store_lock.total_store.list.len() {
                break;
            }

            // TODO: what if pos < curr_left and the loop already break?
            let (curr_is_deleted, curr_content) = {
                let curr_lock = store_lock.total_store.list[left_idx].lock().await;
                (curr_lock.is_deleted, curr_lock.content.clone())
            };

            if !curr_is_deleted {
                pos_limit_left += curr_content.content.len() as i32;
            }
            if pos_limit_left < pos as i32 {
                left_idx += 1;
            } else {
                break;
            }
        }
        println!("-------Current start is : {:?}", left_idx);

        let mut right_idx = left_idx;
        let pos_end = pos + len - 1;
        let mut pos_limit_right = pos_limit_left.clone();
        loop {
            if pos_limit_right < pos_end as i32 {
                right_idx += 1;
            } else {
                break;
            }

            if right_idx >= store_lock.total_store.list.len() {
                return;
            }

            let (curr_is_deleted, curr_content) = {
                let curr_lock = store_lock.total_store.list[right_idx].lock().await;
                (curr_lock.is_deleted, curr_lock.content.clone())
            };

            if !curr_is_deleted {
                pos_limit_right += curr_content.content.len() as i32;
            }
        }
        println!("-------Current end is : {:?}", right_idx);
        {
            println!("-------total_list is : {:?}", store_lock.total_store.list)
        }

        // Delete all blocks in (left_idx, right_idx) directly
        // Delete part or all left_idx and right_idx
        let (start_id, start_content) = {
            let start_lock = store_lock.total_store.list[left_idx].lock().await;
            (start_lock.id.clone(), start_lock.content.clone())
        };
        let (end_id, end_content) = {
            let end_lock = store_lock.total_store.list[right_idx].lock().await;
            (end_lock.id.clone(), end_lock.content.clone())
        };

        if left_idx == right_idx {
            // All texts to be deleted are in the same block
            let block_id = start_id.clone();
            let length = start_content.content.len() as u32;
            // split into three Blocks
            // the middle one will be deleted
            let left_length = length - (pos_limit_left as u32 - pos + 1); // TODO: ?
            let new_block_id;
            if left_length != 0 {
                store_lock.split(block_id.clone(), left_length).await;
                new_block_id = BlockID::new(
                    block_id.client.clone(),
                    block_id.clock.clone() + left_length,
                );
            } else {
                new_block_id = block_id.clone();
            }
            // println!("----- Block id: {:?}", block_id);
            // println!("----- NEW Block id: {:?}", new_blockID);

            if pos_end as i32 == pos_limit_right {
                store_lock.delete(new_block_id).await;
            } else {
                let mid_length = len;
                store_lock.split(new_block_id.clone(), mid_length).await;
                store_lock.delete(new_block_id).await;
            }
        } else {
            // Delete all blocks in between
            let mut i = left_idx + 1;
            while i < right_idx {
                let curr_id = {
                    let curr_lock = store_lock.total_store.list[i].lock().await;
                    curr_lock.id.clone()
                };

                let block_id = curr_id.clone();
                store_lock.delete(block_id).await;
                i += 1;
            }

            // Delete left blocks
            let blk_len = start_content.content.len() as u32;
            let blk_id = start_id.clone();
            let left_length = blk_len - (pos_limit_left as u32 - pos + 1);
            let blk_id_to_del;
            if left_length != 0 {
                store_lock.split(blk_id.clone(), left_length).await;
                blk_id_to_del =
                    BlockID::new(blk_id.client.clone(), blk_id.clock.clone() + left_length);
            } else {
                blk_id_to_del = blk_id.clone();
            }
            store_lock.delete(blk_id_to_del).await;

            // Delete right blocks
            let blk_len = end_content.content.len() as u32;
            let blk_id = end_id.clone();
            let right_length = blk_len - (pos_limit_right as u32 - pos_end);
            if pos_limit_right != pos_end as i32 {
                store_lock.split(blk_id.clone(), right_length).await;
            }
            store_lock.delete(blk_id).await;
        }

        // Update vector clock
        // self.vector_clock.increment(self.client, 1);
    }

    pub async fn to_string(&self) -> String {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        store_lock.to_string().await
    }
}
