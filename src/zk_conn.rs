#![deny(unused_mut)]
extern crate zookeeper;
use crate::utils::ClientID;
use zookeeper::{WatchedEvent, Watcher};

struct RegisterWatcher;

impl Watcher for RegisterWatcher {
    fn handle(&self, e: WatchedEvent) {
        // add this new peer to peer list
        // should include client id and ip address
    }
}

pub struct ZooKeeperConnection {
    zk_addr: String,
}

impl ZooKeeperConnection {
    // given the name of a doc, fetch all the users that have the copy of the doc
    fn register(doc: String, client: ClientID) {}
}
