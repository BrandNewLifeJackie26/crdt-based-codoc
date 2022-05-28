#![deny(unused_mut)]
extern crate zookeeper;
use crate::{
    txn_rpc::{self, txn_service_client::TxnServiceClient},
    utils::{CRDTError, CRDTResult, ClientID, Peer},
};
use std::time::Duration;
use tokio::runtime::Runtime;
use tonic::transport::{Channel, Endpoint};
use zookeeper::{Acl, CreateMode, KeeperState, WatchedEvent, WatchedEventType, Watcher, ZooKeeper};

const ZK_ADDR: &'static str = "127.0.0.1:2181";

struct RegisterWatcher {
    pub channel: Channel,
}

impl RegisterWatcher {
    async fn register_new_user(&self, peers: &Vec<Peer>) {
        let mut client = TxnServiceClient::new(self.channel.clone());
        let peer_list_serialized = serde_json::to_string(&peers);
        if let Ok(peer_list_serialized) = peer_list_serialized {
            let req = tonic::Request::new(txn_rpc::RegisterRequest {
                peer_list: peer_list_serialized,
            });
            let resp = client.sync_peer_list(req).await;
            if let Ok(_) = resp {
                // println!("successfully send new user to peer's list");
            } else {
                // println!("failed to send new user to peer's list");
            }
        } else {
            println!("watcher failed to serialize peer list");
        }
    }
}

impl Watcher for RegisterWatcher {
    fn handle(&self, e: WatchedEvent) {
        match e.event_type {
            WatchedEventType::NodeChildrenChanged => {
                let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);
                if let Ok(zk) = zk {
                    if let Some(path) = e.path {
                        let watch_res = zk.get_children_w(
                            &path[..],
                            RegisterWatcher {
                                channel: self.channel.clone(),
                            },
                        );

                        if let Ok(peers) = watch_res {
                            // find the new child
                            let mut peers_remote = vec![];
                            for peer in peers {
                                let peer_id = peer.parse::<u32>();
                                match peer_id {
                                    Ok(peer_id) => {
                                        let child_path = format!("{}/{}", path, peer);
                                        let ip_addr_res = zk.get_data(&child_path[..], false);
                                        if let Ok(ip_addr) = ip_addr_res {
                                            let ip_addr = String::from_utf8(ip_addr.0.clone());
                                            match ip_addr {
                                                Ok(ip_addr) => {
                                                    peers_remote.push(Peer {
                                                        client_id: peer_id,
                                                        ip_addr: ip_addr,
                                                    });
                                                }
                                                Err(e) => {
                                                    println!("can't convert string {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => println!("dummy node, ignore"),
                                }
                            }
                            // call into transaction's register function
                            Runtime::new()
                                .unwrap()
                                .block_on(self.register_new_user(&peers_remote));
                        }
                    } else {
                        println!("register watcher cannot initialize");
                    }
                }
            }
            _ => println!("unsupported event type"),
        }
    }
}

struct DefaultWatcher;
impl Watcher for DefaultWatcher {
    fn handle(&self, e: WatchedEvent) {
        match e.keeper_state {
            KeeperState::SyncConnected => {
                println!("successfully connect to zookeeper");
            }
            KeeperState::AuthFailed => {
                println!("failed to authenticate");
            }
            KeeperState::Disconnected => {
                println!("disconnected from the server");
            }
            _ => {
                println!("zookeeper connection state that shouldn't be happening");
            }
        }
    }
}

pub struct ZooKeeperConnection {
    pub client_ip: String,
}

impl ZooKeeperConnection {
    pub async fn background_sync(&self, doc: String) {
        let path = format!("/{}", doc);
        let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);
        if let Ok(zk) = zk {
            let http_path = format!("http://{}", self.client_ip.clone());
            let endpoint = Endpoint::from_shared(http_path);
            match endpoint {
                Ok(ep) => {
                    let temp = ep.connect().await;
                    match temp {
                        Ok(ch) => {
                            let _ = zk.get_children_w(&path[..], RegisterWatcher { channel: ch });
                        }
                        Err(e) => println!("zookeepeer failed to connect to local node: {:?}", e),
                    }
                }
                Err(_) => println!("zookeepeer failed to connect to endpoint"),
            }
        } else {
            println!("failed to start zookeeper");
        }
    }

    // given the name of a doc, fetch all the users that have the copy of the doc
    pub async fn register(&self, doc: String, client: ClientID) -> CRDTResult<Vec<Peer>> {
        let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);

        match zk {
            Ok(zk) => {
                println!("connected to {:?}", ZK_ADDR);
                let path = format!("/{}", doc);
                let mut peers_remote = vec![];

                // check if doc path exist
                let exists = zk.exists(&path[..], false);
                match exists {
                    Ok(exists) => {
                        if let None = exists {
                            // this file does not exist, create one
                            println!("creating the doc directory");
                            let create_res = zk.create(
                                &path[..],
                                "".as_bytes().to_vec(),
                                Acl::open_unsafe().clone(),
                                CreateMode::Persistent,
                            );
                            match create_res {
                                Ok(_) => {
                                    // println!("successfully create doc directory");
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                    // return Err(Box::new(CRDTError::ZKCreateZnodeFailed(path)));
                                }
                            }
                        }

                        // create the child node
                        let child_path = format!("/{}/{}", doc, client);
                        println!("the child path is {:?}", child_path);
                        let res = zk.create(
                            &child_path[..],
                            self.client_ip.as_bytes().to_vec(),
                            Acl::open_unsafe().clone(),
                            CreateMode::Persistent,
                        );
                        match res {
                            Ok(_) => {
                                println!("successfully created node for this client");
                            }
                            Err(e) => {
                                println!("cannot create node for this client because {:?}", e);
                                return Err(Box::new(CRDTError::ZKCreateZnodeFailed(child_path)));
                            }
                        }

                        let watch_res = zk.get_children(&path[..], false);
                        if let Ok(full_peer_list) = watch_res {
                            for peer in full_peer_list {
                                let peer_id = peer.parse::<u32>();
                                match peer_id {
                                    Ok(peer_id) => {
                                        let child_path = format!("{}/{}", path, peer);
                                        let ip_addr_res = zk.get_data(&child_path[..], false);
                                        if let Ok(ip_addr) = ip_addr_res {
                                            peers_remote.push(Peer {
                                                client_id: peer_id,
                                                ip_addr: String::from_utf8(ip_addr.0.clone())
                                                    .unwrap(),
                                            });
                                        }
                                    }
                                    Err(_) => println!("invalid client id"),
                                }
                            }
                            return Ok(peers_remote);
                        } else {
                            println!("zookeepeer failed to watch the doc path");
                        }
                    }
                    Err(_) => println!("zookeeper failed to call exist"),
                }
            }
            Err(_) => println!("failed to connect to zk"),
        }
        Err(Box::new(CRDTError::RegisterUserFailed()))
    }
}
