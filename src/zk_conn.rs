#![deny(unused_mut)]
extern crate zookeeper;
use crate::{
    txn_rpc::{self, txn_service_client::TxnServiceClient},
    utils::{CRDTError, CRDTResult, ClientID, Peer},
};
use std::time::Duration;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{channel, Receiver, Sender},
};
use tonic::transport::{Channel, Endpoint};
use zookeeper::{Acl, CreateMode, KeeperState, WatchedEvent, WatchedEventType, Watcher, ZooKeeper};

const ZK_ADDR: &'static str = "127.0.0.1:2181";

struct RegisterWatcher {
    pub channel: Channel,
    pub sender: Sender<()>,
    pub id: u32, // used for debugging
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
            match resp {
                Ok(_) => println!("successfully send new user to peer's list"),
                Err(e) => println!("failed to send new user to peer's list {:?}", e),
            }
        } else {
            println!("watcher failed to serialize peer list");
        }
    }
}

impl Watcher for RegisterWatcher {
    fn handle(&self, e: WatchedEvent) {
        let mut peers_remote = vec![];
        match e.event_type {
            WatchedEventType::NodeChildrenChanged => {
                if let Some(path) = e.path {
                    let _ = Runtime::new().unwrap().block_on(self.sender.send(()));
                    let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);
                    if let Ok(zk) = zk {
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
                            Runtime::new()
                                .unwrap()
                                .block_on(self.register_new_user(&peers_remote));
                        } else {
                            println!("can't retrieve full peer list");
                        }
                    }
                }
            }
            _ => println!("unsupported event type"),
        }
    }
}

struct DefaultWatcher;
impl Watcher for DefaultWatcher {
    fn handle(&self, _: WatchedEvent) {
        // just a placeholder
    }
}

pub struct ZooKeeperConnection {
    pub client_ip: String,
}

impl ZooKeeperConnection {
    pub async fn background_sync(&self, doc: String, sender: Sender<()>) {
        println!("backend process started!");
        let path = format!("/{}", doc);
        let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);
        let (sender_block, mut receiver_block): (Sender<()>, Receiver<()>) = channel(1);

        if let Ok(zk) = zk {
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
                        if let Err(e) = create_res {
                            println!("{:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                    // return Err(Box::new(CRDTError::ZKCreateZnodeFailed(path)));
                }
            }

            // trigger user service to start
            let _ = sender.send(()).await;

            loop {
                let http_path = format!("http://{}", self.client_ip.clone());
                let endpoint = Endpoint::from_shared(http_path);
                match endpoint {
                    Ok(ep) => {
                        let temp = ep.connect().await;
                        match temp {
                            Ok(ch) => {
                                let _ = zk.get_children_w(
                                    &path[..],
                                    RegisterWatcher {
                                        channel: ch,
                                        sender: sender_block.clone(),
                                        id: 0,
                                    },
                                );
                            }
                            Err(e) => {
                                println!("zookeepeer failed to connect to local node: {:?}", e)
                            }
                        }
                    }
                    Err(_) => println!("zookeepeer failed to connect to endpoint"),
                }
                let _ = receiver_block.recv().await;
            }
        } else {
            println!("failed to start zookeeper");
        }
        println!("background shutting down");
    }

    // given the name of a doc, fetch all the users that have the copy of the doc
    pub async fn register(&self, doc: String, client: ClientID) -> CRDTResult<Vec<Peer>> {
        let zk = ZooKeeper::connect(&*ZK_ADDR, Duration::from_secs(15), DefaultWatcher);

        match zk {
            Ok(zk) => {
                println!("connected to {:?}", ZK_ADDR);
                let path = format!("/{}", doc);
                let mut peers_remote = vec![];

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
                                        ip_addr: String::from_utf8(ip_addr.0.clone()).unwrap(),
                                    });
                                }
                            }
                            Err(_) => println!("invalid client id"),
                        }
                    }
                    return Ok(peers_remote);
                }
            }
            Err(_) => println!("failed to connect to zk"),
        }
        Err(Box::new(CRDTError::RegisterUserFailed()))
    }
}
