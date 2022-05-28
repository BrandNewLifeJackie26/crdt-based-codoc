use crate::doc::Doc;
use crate::doc::VectorClock;
use crate::txn_rpc;
use crate::txn_rpc::txn_service_client::TxnServiceClient;
use crate::txn_rpc::txn_service_server::TxnService;
use crate::utils::Peer;
use crate::utils::{ClientID, Updates};
use crate::zk_conn::ZooKeeperConnection;
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
    pub doc_name: String,
    pub doc: Arc<Mutex<Doc>>,
    pub channels: Arc<Mutex<HashMap<ClientID, Channel>>>,
    // zookeeper utils
    pub zk: ZooKeeperConnection,
    // unique identifier for this client
    pub client: ClientID,
    pub client_ip: String,
}

impl SyncTransaction {
    pub fn new(
        doc_name: String,
        client: ClientID,
        doc: Arc<Mutex<Doc>>,
        channels: Arc<Mutex<HashMap<ClientID, Channel>>>,
        client_ip: String,
    ) -> Self {
        SyncTransaction {
            doc_name: doc_name,
            doc: doc,
            channels: channels,
            client: client,
            client_ip: client_ip.clone(),
            zk: ZooKeeperConnection {
                client_ip: client_ip,
            },
        }
    }

    // request all updates from its peers and deduplicate
    // resolve all conflicts
    pub async fn sync(&self) {
        // get all the peers that are editing the same doc
        let mut real_channel = self.channels.lock().await;
        let peers;
        {
            let real_doc = self.doc.lock().await;
            peers = real_doc.peers.clone();
        }

        // for all peers call on rpc to get all updates
        for client in peers.into_iter() {
            if client.client_id == client.client_id {
                continue;
            }
            // if connection already established, reuse the connection
            let conn = real_channel.get(&client.client_id);
            match conn {
                Some(_) => {
                    // just reuse
                }
                None => {
                    let http_path = format!("http://{}", client.ip_addr);
                    let endpoint = Endpoint::from_shared(http_path);
                    if let Ok(ep) = endpoint {
                        let temp = ep.connect().await;
                        if let Ok(ch) = temp {
                            (*real_channel).insert(client.client_id, ch);
                        }
                    }
                }
            }

            if let Some(new_channel) = real_channel.get(&client.client_id) {
                let mut client = TxnServiceClient::new(new_channel.clone());
                let local_doc = self.doc.lock().await;
                let clock_serialized = serde_json::to_string(&local_doc.vector_clock);
                match clock_serialized {
                    // serialize the local vector clock send our through rpc
                    Ok(clock_serialized) => {
                        let req = tonic::Request::new(txn_rpc::PullRequest {
                            client_id: self.client,
                            vector_clock: clock_serialized,
                        });
                        let resp = client.get_remote_updates(req).await;
                        match resp {
                            Ok(value) => {
                                let remote_updates: Result<Updates, serde_json::Error> =
                                    serde_json::from_str(&value.into_inner().updates);
                                match remote_updates {
                                    Ok(remote_updates) => {
                                        self.update_remote(remote_updates).await;
                                    }
                                    Err(_) => println!("serde deserialization error"),
                                }
                            }
                            Err(_) => println!("rpc error"),
                        };
                    }
                    Err(_) => println!("serde serialization error"),
                }
            }
        }
    }

    // update peers' modifications on local copy
    // don't need to deal with conflicts
    pub async fn update_remote(&self, updates: Updates) {
        let mut delete_list: Updates = vec![];
        let mut update_list: Updates = vec![];
        for update in updates {
            if update.is_deleted {
                delete_list.push(update);
            } else {
                update_list.push(update);
            }
        }

        let mut local_doc = self.doc.lock().await;
        local_doc.insert_remote(update_list).await;
        local_doc.delete_remote(delete_list).await;
    }

    // takes in a vector clock, compare with its own vector clock,
    // compute updates that need to be send
    pub async fn compute_diff(&self, remote_clocks: VectorClock) -> Updates {
        // get its own state vector
        let local_clocks;
        {
            let local_doc = self.doc.lock().await;
            local_clocks = local_doc.vector_clock.clone();
        }
        let mut res: Updates = vec![];

        // compute the difference
        for (client_id, local_clock) in local_clocks.clock_map.into_iter() {
            let remote_clock = remote_clocks.clock_map.get(&client_id);
            match remote_clock {
                Some(remote_clock) => {
                    if *remote_clock < local_clock {
                        // need to send the remaining part to the counterpart
                        res.extend(self.construct_updates(*remote_clock + 1, client_id).await);
                    }
                }
                None => {
                    // need to forword all they have to the requester
                    res.extend(self.construct_updates(0, client_id).await);
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
                            res.push((**b).clone());
                        }
                    }
                    None => {
                        // should not happen
                    }
                }
            }
            None => {
                // should not happen
            }
        }
        return res;
    }

    // consult zookeeper and sync with other peers when started
    pub async fn register(&self) -> bool {
        let mut local_doc = self.doc.lock().await;
        let reg_res = self.zk.register(local_doc.name.clone(), self.client).await;
        if let Ok(reg_res) = reg_res {
            println!(
                "{:?} successfully get the peer list: {:?}",
                self.client, reg_res
            );
            local_doc.peers = reg_res;
            return true;
        }
        false
    }
}

// implement rpc interface
#[async_trait::async_trait]
impl TxnService for SyncTransaction {
    async fn get_remote_updates(
        &self,
        request: tonic::Request<txn_rpc::PullRequest>,
    ) -> Result<tonic::Response<txn_rpc::PullResponse>, tonic::Status> {
        let temp_request = request.into_inner();
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

    async fn sync_peer_list(
        &self,
        request: tonic::Request<txn_rpc::RegisterRequest>,
    ) -> Result<tonic::Response<txn_rpc::Status>, tonic::Status> {
        println!("{:?} received new node added notification", self.client);
        let temp_request = request.into_inner();
        let peers_remote_res: Result<Vec<Peer>, serde_json::Error> =
            serde_json::from_str(&temp_request.peer_list);
        if let Ok(peers_remote) = peers_remote_res {
            println!(
                "{:?} successfully received up-to-dated peer list {:?}",
                self.client, peers_remote
            );
            let mut local_doc = self.doc.lock().await;
            let peers_local = local_doc.peers.clone();
            for client in peers_remote {
                if !peers_local.contains(&client) {
                    // this is the new user
                    local_doc.peers.push(Peer {
                        client_id: client.client_id,
                        ip_addr: client.ip_addr,
                    });
                }
            }
            return Ok(tonic::Response::new(txn_rpc::Status { succ: true }));
        } else {
            return Err(tonic::Status::invalid_argument("rpc error"));
        }
    }
}
