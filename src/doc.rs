use crate::utils::{ClientID, Peer, Updates};
use crate::{block::Content, block_store::BlockStore, Block, BlockID};
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

    fn increment(&mut self, client: ClientID) {
        self.clock_map.insert(
            client,
            self.clock_map.get(&client).cloned().unwrap_or(0) + 1,
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
    pub async fn insert_remote(&mut self, update: Updates) {
        for block in update.iter() {
            // Try insert pending updates
            self.flush_pending_updates().await;
            // Try insert current updates
            let success = self.insert_single_block(block).await;
            if !success {
                self.pending_updates.push(block.clone());
            }
        }
    }

    pub async fn insert_single_block(&mut self, block: &Block) -> bool {
        // Try insert, return false if failed, return true if success
        // First find the block corresponding the left_origin and right_origin

        let left_res = self
            .find_block_idx(block.left_origin.clone(), 0, true)
            .await;
        if let Err(e) = left_res {
            // not exist
            return false;
        }
        let left = left_res.unwrap();
        let right_res = self
            .find_block_idx(block.right_origin.clone(), left, false)
            .await;
        if let Err(e) = right_res {
            // not exist
            return false;
        }
        let right = right_res.unwrap();
        let mut i = (left + 1) as usize;
        let mut scan = false;
        let mut dest = (left + 1) as usize;

        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        loop {
            if !scan {
                dest = i;
            }
            if i == store_lock.total_store.list.len() || i == (right as usize) {
                break;
            }
            let curr = &store_lock.total_store.list[i];
            let curr_ol = self
                .find_block_idx(curr.left_origin.clone(), 0, true)
                .await
                .unwrap();
            let curr_or = self
                .find_block_idx(curr.right_origin.clone(), curr_ol, false)
                .await
                .unwrap();

            if curr_ol < left {
                break;
            } else if curr_ol == left {
                if curr_or < right {
                    scan = true;
                    continue;
                } else if curr_or == right {
                    if block.id < curr.id {
                        break;
                    } else {
                        scan = false;
                        continue;
                    }
                } else {
                    scan = false;
                    continue;
                }
            } else {
                continue;
            }
        }
        let new_block = block.clone();
        let left_id;
        if dest == 0 {
            left_id = None;
        } else {
            left_id = Some(store_lock.total_store.list[dest - 1].id.clone());
        }
        store_lock.insert(new_block, left_id);
        true
    }

    pub async fn delete_single_block(&mut self, block: &Block) -> bool {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        let mut i = 0;
        let id = block.id.clone();
        while i < store_lock.total_store.list.len() {
            if store_lock.total_store.list[i].id == id {
                store_lock.delete(block.id.clone());
                return true;
            } else if store_lock.total_store.list[i].id.client == id.client
                && store_lock.total_store.list[i].id.clock < id.clock
                && store_lock.total_store.list[i].id.clock
                    + store_lock.total_store.list[i].content.content.len() as u32
                    - 1
                    >= id.clock
            {
                let block_to_split = store_lock.total_store.list[i].id.clone();
                let len = id.clock - store_lock.total_store.list[i].id.clock;
                store_lock.split(block_to_split, len);
                store_lock.delete(block.id.clone());
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
                    if store_lock.total_store.list[i].id == id {
                        return Ok(i as i64);
                    } else if store_lock.total_store.list[i].id.client == id.client
                        && store_lock.total_store.list[i].id.clock < id.clock
                        && store_lock.total_store.list[i].id.clock
                            + store_lock.total_store.list[i].content.content.len() as u32
                            - 1
                            >= id.clock
                    {
                        let block_to_split = store_lock.total_store.list[i].id.clone();
                        let len = id.clock - store_lock.total_store.list[i].id.clock;
                        store_lock.split(block_to_split, len);
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

        while prev_char_cnt < pos && idx < (*store_lock).total_store.list.len() {
            // TODO: what if pos < curr and the loop already break?
            if !(*store_lock).total_store.list[idx].is_deleted {
                prev_char_cnt += (*store_lock).total_store.list[idx].content.content.len() as u32;
            }
            idx += 1;
        }

        let new_block_clk;
        if store_lock.kv_store.contains_key(&self.client) {
            let list = store_lock.kv_store.get(&self.client).unwrap().list.clone();
            if list.len() == 0 {
                new_block_clk = 0;
            } else {
                new_block_clk = list.last().unwrap().id.clock
                    + list.last().unwrap().content.content.len() as u32;
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
        if idx == (*store_lock).total_store.list.len()
            && pos >= (*store_lock).to_string().len() as u32
        {
            // Append to the end
            if idx > 0 {
                let left_id = Some((*store_lock).total_store.list[idx - 1].id.clone());
                new_block.left_origin = left_id.clone();
                (*store_lock).insert(new_block, left_id);
            } else {
                (*store_lock).insert(new_block, None);
            }
        } else if prev_char_cnt == pos {
            // Insert to i-th position in total_store
            let left_id = Some((*store_lock).total_store.list[idx - 1].id.clone());
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some((*store_lock).total_store.list[idx].id.clone());
            (*store_lock).insert(new_block, left_id);
        } else {
            // Have to split total_store[i-1]
            let left_id = Some((*store_lock).total_store.list[idx - 1].id.clone());
            let left_content_len = (*store_lock).total_store.list[idx - 1]
                .content
                .content
                .len() as u32;
            new_block.left_origin = left_id.clone();
            new_block.right_origin = Some(BlockID::new(
                left_id.clone().unwrap().client,
                left_id.clone().unwrap().clock + left_content_len - (prev_char_cnt - pos),
            ));

            // Split the block
            (*store_lock).split(
                left_id.clone().unwrap(),
                left_content_len - (prev_char_cnt - pos),
            );
            // println!(
            //     "----- AFTER SPLIT {:?} -----",
            //     (*store_lock).total_store.list
            // );
            (*store_lock).insert(new_block, left_id);
            // println!(
            //     "----- AFTER INSERT {:?} -----",
            //     (*store_lock).total_store.list
            // );
        }

        // Update vector clock
        self.vector_clock.increment(self.client);
    }

    // Delete the content of length len from pos
    pub async fn delete_remote(&mut self, update: Updates) {
        for block in update.iter() {
            // Try insert pending updates
            self.flush_pending_updates().await;
            // Try insert current updates
            let success = self.delete_single_block(block).await;
            if !success {
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
            }
        }
    }

    pub async fn delete_local(&mut self, pos: u32, len: u32) {
        let store = self.block_store.clone();
        let mut store_lock = store.lock().await;

        // Find the correct blocks to delete
        // The block may need to be splitted
        let mut i_start = 0 as usize;
        let mut curr_start = -1;

        loop {
            // TODO: what if pos < curr_left and the loop already break?
            if !(*store_lock).total_store.list[i_start].is_deleted {
                curr_start += (*store_lock).total_store.list[i_start]
                    .content
                    .content
                    .len() as i32;
            }
            if curr_start < pos as i32 {
                i_start += 1;
            } else {
                break;
            }
        }
        // println!("-------Current start is : {:?}", i_start);

        let mut i_end = i_start;
        let mut curr_end = curr_start;
        let mut pos_end = pos + len - 1;
        loop {
            if curr_end < pos_end as i32 {
                i_end += 1;
            } else {
                break;
            }
            if !(*store_lock).total_store.list[i_end].is_deleted {
                curr_end += (*store_lock).total_store.list[i_end].content.content.len() as i32;
            }
        }
        // println!("-------Current end is : {:?}", i_end);

        // Delete all blocks in (i_start, i_end) directly
        // Delete i_start and i_end according to the position
        if i_start == i_end {
            // All texts to be deleted are in the same block
            let block_id = (*store_lock).total_store.list[i_start].id.clone();
            let length = (*store_lock).total_store.list[i_start]
                .content
                .content
                .len() as u32;
            // split into three Blocks
            // the middle one will be deleted
            let left_length = length - curr_start as u32 + pos - 1; // TODO: ?
                                                                    // println!("------- Left length: {:?}", left_length);
            let mut new_blockID;
            if left_length != 0 {
                (*store_lock).split(block_id.clone(), left_length);
                new_blockID = BlockID::new(
                    block_id.client.clone(),
                    block_id.clock.clone() + left_length,
                );
            } else {
                new_blockID = block_id.clone();
            }
            // println!("----- Block id: {:?}", block_id);
            // println!("----- NEW Block id: {:?}", new_blockID);

            if pos_end as i32 == curr_start {
                (*store_lock).delete(new_blockID);
            } else {
                let mid_length = len;
                store_lock.split(new_blockID.clone(), mid_length);
                // println!("------ list {:?}", (*store_lock).total_store.list);
                // println!(
                //     "------ map {:?}",
                //     (*store_lock).kv_store.get(&1).unwrap().list
                // );
                // println!(
                //     "------ block map {:?}",
                //     (*store_lock).block_map.get(&new_blockID.clone())
                // );
                (*store_lock).delete(new_blockID.clone());
                // println!("------ list {:?}", (*store_lock).total_store.list);
                // println!(
                //     "------ map {:?}",
                //     (*store_lock).kv_store.get(&1).unwrap().list
                // );
                // println!(
                //     "------ block map {:?}",
                //     (*store_lock).block_map.get(&new_blockID.clone())
                // );
            }
        } else {
            // Delete all blocks in between
            let mut i = i_start + 1;
            while i < i_end {
                let block_id = (*store_lock).total_store.list[i].id.clone();
                (*store_lock).delete(block_id);
            }

            // Delete left blocks
            let length_start = (*store_lock).total_store.list[i_start]
                .content
                .content
                .len() as u32;
            let block_id_start = (*store_lock).total_store.list[i_start].id.clone();
            let left_length = length_start - curr_start as u32 + pos - 1;
            let mut new_blockID;
            if left_length != 0 {
                (*store_lock).split(block_id_start.clone(), left_length);
                new_blockID = BlockID::new(
                    block_id_start.client.clone(),
                    block_id_start.clock.clone() + left_length,
                );
            } else {
                new_blockID = block_id_start.clone();
            }
            (*store_lock).delete(new_blockID);

            // Delete right blocks
            let length_end = (*store_lock).total_store.list[i_end].content.content.len() as u32;
            let block_id_end = (*store_lock).total_store.list[i_end].id.clone();
            let right_length = length_end - curr_end as u32 + pos;
            if curr_end != pos_end as i32 {
                (*store_lock).split(block_id_end.clone(), right_length);
            }
            (*store_lock).delete(block_id_end);
        }

        // Update vector clock
        self.vector_clock.increment(self.client);
    }

    pub async fn to_string(&self) -> String {
        let store = self.block_store.clone();
        let store_lock = store.lock().await;
        (*store_lock).to_string()
    }
}
