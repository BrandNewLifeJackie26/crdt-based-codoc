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

    pub async fn get_content(&self) -> String {
        self.sync().await;

        let doc = self.doc.lock().await;
        doc.to_string().await
    }

    // request all updates from its peers and deduplicate
    // resolve all conflicts
    pub async fn sync(&self) {
        // get all the peers that are editing the same doc
        println!("[sync_txn] {:?} start syncing...", self.client);
        let mut real_channel = self.channels.lock().await;
        let peers;
        {
            let real_doc = self.doc.lock().await;
            peers = real_doc.peers.clone();
        }

        // for all peers call on rpc to get all updates
        for client in peers.into_iter() {
            if client.client_id == self.client {
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
                let clock_serialized;
                {
                    let local_doc = self.doc.lock().await;
                    clock_serialized = serde_json::to_string(&local_doc.vector_clock);
                }
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
                                let resp = value.into_inner();
                                let remote_updates: Result<
                                    HashMap<u32, Updates>,
                                    serde_json::Error,
                                > = serde_json::from_str(&resp.updates);
                                let peer_id = resp.client_id;
                                match remote_updates {
                                    Ok(remote_updates) => {
                                        println!(
                                            "[sync_txn_sync] {:?} successfully get all updates",
                                            self.client
                                        );
                                        for (keypeer_id, ups) in remote_updates.into_iter() {
                                            self.update_remote(keypeer_id, ups).await;
                                        }
                                    }
                                    Err(_) => {
                                        println!("[sync_txn_sync] serde deserialization error")
                                    }
                                }
                            }
                            Err(_) => println!("[sync_txn_sync] rpc error"),
                        };
                    }
                    Err(_) => println!("[sync_txn_sync] serde serialization error"),
                }
            }
        }
        println!("[sync_txn] {:?} sync fin", self.client);
    }

    // update peers' modifications on local copy
    // don't need to deal with conflicts
    pub async fn update_remote(&self, peer_id: u32, updates: Updates) {
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
        {
            local_doc.insert_remote(update_list, peer_id).await;
            local_doc.delete_remote(delete_list, peer_id).await;
        }
    }

    // takes in a vector clock, compare with its own vector clock,
    // compute updates that need to be send
    pub async fn compute_diff(&self, remote_clocks: VectorClock) -> HashMap<u32, Updates> {
        // get its own state vector
        let local_clocks;
        {
            let local_doc = self.doc.lock().await;
            local_clocks = local_doc.vector_clock.clone();
            println!(
                "[sync_txn_cf] {:?} local clock is {:?}",
                self.client, local_clocks
            );
            println!(
                "[sync_txn_cf] {:?} remote clock is {:?}",
                self.client, remote_clocks
            );
        }
        let mut t_res = HashMap::new();

        // compute the difference
        for (client_id, local_clock) in local_clocks.clock_map.into_iter() {
            let mut res: Updates = vec![];
            let remote_clock = remote_clocks.clock_map.get(&client_id);
            match remote_clock {
                Some(remote_clock) => {
                    if *remote_clock < local_clock {
                        // need to send the remaining part to the counterpart
                        res.extend(self.construct_updates(*remote_clock, client_id).await);
                    }
                }
                None => {
                    // need to forword all they have to the requester
                    res.extend(self.construct_updates(0, client_id).await);
                }
            }
            t_res.insert(client_id, res);
        }

        println!(
            "[sync_txn_compute_diff] {:?} successfully computed all diffs",
            self.client
        );
        return t_res;
    }

    // given a diff range, consult the block store and find all the updates
    // need to send to the counterpart
    async fn construct_updates(&self, start: u32, client: ClientID) -> Updates {
        // go to block store and get the updates
        println!(
            "[sync_txn_cu] {:?} for {:?} start from {:?}",
            self.client, client, start
        );
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
                            let b_lock = b.lock().await;
                            res.push(b_lock.clone());
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
        println!("[sync_txn_cu] {:?} succ", self.client);
        return res;
    }

    // consult zookeeper and sync with other peers when started
    pub async fn register(&self) -> bool {
        let reg_res = self.zk.register(self.doc_name.clone(), self.client).await;

        match reg_res {
            Err(e) => {
                println!(
                    "[sync_txn_register] {:?} register user failed because of {:?}",
                    self.client, e
                );
                return false;
            }
            Ok(res) => {
                let mut temp = self.doc.lock().await;
                temp.vector_clock.set(self.client, 0);
                temp.peers = res;
                println!(
                    "[sync_txn_register]{:?} successfully received up-to-dated peer list {:?}",
                    self.client, temp.peers
                );
                return true;
            }
        }
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
                            client_id: self.client,
                        }))
                    }
                    Err(_) => {
                        return Err(tonic::Status::invalid_argument(
                            "[sync_txn] serialized rpc error",
                        ))
                    }
                }
            }
            Err(_) => {
                return Err(tonic::Status::invalid_argument(
                    "[sync_txn] deserialized rpc error",
                ))
            }
        }
    }

    async fn sync_peer_list(
        &self,
        request: tonic::Request<txn_rpc::RegisterRequest>,
    ) -> Result<tonic::Response<txn_rpc::Status>, tonic::Status> {
        println!(
            "[sync_txn] {:?} received new node added notification",
            self.client
        );
        let temp_request = request.into_inner();
        let peers_remote_res: Result<Vec<Peer>, serde_json::Error> =
            serde_json::from_str(&temp_request.peer_list);
        if let Ok(peers_remote) = peers_remote_res {
            println!(
                "[sync_txn] {:?} successfully received up-to-dated peer list {:?}",
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
