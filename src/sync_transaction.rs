use crate::block::Block;
use crate::doc::Doc;
use crate::doc::VectorClock;
use crate::txn_rpc;
use crate::txn_rpc::txn_service_client::TxnServiceClient;
use crate::txn_rpc::txn_service_server::TxnService;
use crate::utils::{ClientID, Updates};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

// SyncTransaction is used to sync updates (insertion and deletion) among different clients
//
// IMPORTANT: SyncTransaction will take in a created Doc and modify its states
pub struct SyncTransaction {
    // local copy of the doc
    doc: Arc<Mutex<Doc>>,
    channels: Arc<Mutex<HashMap<ClientID, Channel>>>,

    // unique identifier for a client
    client: ClientID,
}

impl SyncTransaction {
    pub fn new(client: ClientID, doc: Arc<Mutex<Doc>>) -> Self {
        SyncTransaction {
            doc: doc,
            channels: Arc::new(Mutex::new(HashMap::new())),
            client: client,
        }
    }

    // request all updates from its peers and deduplicate
    // resolve all conflicts
    pub async fn sync(&self) {
        // for all peers call on rpc to get all updates
        let mut real_channel = self.channels.lock().await;
        let peers;
        {
            let real_doc = self.doc.lock().await;
            peers = real_doc.peers.clone();
        }
        for client in peers.into_iter() {
            let conn = real_channel.get(&client.client_id);
            match conn {
                Some(conn) => {
                    // just reuse
                }
                None => {
                    // todo: look up the address of the peer
                    let endpoint = Endpoint::from_shared("ip addess");
                    if let Ok(ep) = endpoint {
                        let temp = ep.connect().await;
                        if let Ok(res) = temp {
                            (*real_channel).insert(client.client_id, res);
                        }
                    }
                }
            }

            if let Some(new_channel) = real_channel.get(&client.client_id) {
                let mut client = TxnServiceClient::new(new_channel.clone());
                let vector_clock = self.doc.lock().await;
                let clock_serialized = serde_json::to_string(&vector_clock.vector_clock);
                match clock_serialized {
                    Ok(clock_serialized) => {
                        let req = tonic::Request::new(txn_rpc::PullRequest {
                            client_id: self.client,
                            vector_clock: clock_serialized,
                        });
                        let resp = client.get_remote_updates(req).await;
                        match resp {
                            Ok(value) => {
                                // TODO: start resolve conflict:
                            }
                            Err(_) => println!("rpc error"),
                        };
                    }
                    Err(_) => println!("serialization error"),
                }
            }
        }
    }

    // update peers' modifications on local copy
    // don;t need to deal with conflicts
    pub fn update_remote(&self, updates: Updates) {
        let mut delete_list: Vec<Block> = vec![];
        for update in updates {
            if update.is_deleted {
                delete_list.push(update);
            } else {
                // call on doc
            }
        }
    }

    // obtain a delete set from update_remote(), and apply peer deletions
    pub fn delete_remote(&mut self, mut update: Updates) {}

    // takes in a vector clock, compare with its own vector clock,
    // compute updates that need to be send
    async fn compute_diff(&self, remote_clocks: VectorClock) -> Updates {
        // get its own state vector
        let clocks;
        {
            let local_doc = self.doc.lock().await;
            clocks = local_doc.vector_clock.clone();
        }

        let mut res: Updates = vec![];
        // compute the difference
        for (key, local_clock) in clocks.clock_map.into_iter() {
            let remote_clock = remote_clocks.clock_map.get(&key);
            match remote_clock {
                Some(remote_clock) => {
                    if *remote_clock < local_clock {
                        // need to send the remaining part to the counterpart
                        res.extend(self.construct_updates(*remote_clock + 1, key).await);
                    }
                }
                None => {
                    // need to forword all they have to the requester
                    res.extend(self.construct_updates(0, key).await);
                }
            }
        }

        return res;
    }

    // given a diff range, consult the block store and find all the updates
    // need to send to the counterpart
    async fn construct_updates(&self, start: u32, client: ClientID) -> Updates {
        // go to block store and get the updates
        let local_doc = self.doc.lock().await;
        let block_store = local_doc.block_store.lock().await;
        let list = block_store.kv_store.get(&client).clone();

        let mut res: Updates = vec![];
        match list {
            Some(list) => {
                let block = list.list.get((start as usize)..list.list.len());
                match block {
                    Some(block) => {
                        let vec = block.iter();
                        for b in vec {
                            res.push(b.clone());
                        }
                    }
                    None => {
                        // should not happen
                    }
                }
            }
            None => {}
        }
        return res;
    }

    // consult zookeeper and sync with other peers when started
    pub fn register(client: ClientID) {}

    // the callback function called by zookeeper
    pub fn register_call_back(client: ClientID) {}
}

// implement rpc interface
#[async_trait::async_trait]
impl TxnService for SyncTransaction {
    async fn get_remote_updates(
        &self,
        request: tonic::Request<txn_rpc::PullRequest>,
    ) -> Result<tonic::Response<txn_rpc::PullResponse>, tonic::Status> {
        let temp_request = request.into_inner();
        let client_id = temp_request.client_id;
        let vector_string = temp_request.vector_clock;

        let vector_clock = serde_json::from_str::<VectorClock>(&vector_string);
        match vector_clock {
            Ok(vector_clock) => {
                let updates = self.compute_diff(vector_clock).await;
                let updates_serialized = serde_json::to_string(&updates);
                match updates_serialized {
                    Ok(updates_serialized) => {
                        return Ok(tonic::Response::new(txn_rpc::PullResponse {
                            updates: updates_serialized,
                        }))
                    }
                    Err(_) => return Err(tonic::Status::invalid_argument("serialized rpc error")),
                }
            }
            Err(_) => return Err(tonic::Status::invalid_argument("deserialized rpc error")),
        }
    }
}
