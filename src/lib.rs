pub mod block;
pub mod block_store;
pub mod doc;
pub mod list;
pub mod sync_transaction;
pub mod txn_rpc;
pub mod utils;
pub mod zk_conn;
pub use crate::block::Block;
pub use crate::block::BlockID;
pub use crate::block_store::BlockStore;

#[cfg(test)]
mod local_tests {
    use crate::block::{Block, BlockID};
    use crate::block_store;
    use crate::doc::Doc;
    use crate::list::*;
    use crate::utils::ClientID;

    // Local insert to a single doc,
    // there is no need to use transaction if no sync is needed
    #[test]
    fn local_insert() {
        let cid = 1 as ClientID;
        let doc = Doc::new("text".to_string(), cid);

        // let uid = UID { id: 1 };
        // let text = List::new(uid.clone());

        // let start_item_id = ItemID { id: ID_START };
        // let end_item_id = ItemID { id: ID_END };

        // let item_id1 = text.insert(
        //     start_item_id.clone(),
        //     end_item_id.clone(),
        //     Content {
        //         content: "1".to_string(),
        //     },
        // );
        // assert_eq!(text.to_string(), "1".to_string());

        // let item_id2 = text.insert(
        //     start_item_id.clone(),
        //     item_id1.clone(),
        //     Content {
        //         content: "2".to_string(),
        //     },
        // );
        // assert_eq!(text.to_string(), "21".to_string());

        // let item_id3 = text.insert(
        //     item_id1.clone(),
        //     end_item_id.clone(),
        //     Content {
        //         content: "3".to_string(),
        //     },
        // );
        // assert_eq!(text.to_string(), "213".to_string());

        // let item_id4 = text.insert(
        //     item_id1.clone(),
        //     item_id3.clone(),
        //     Content {
        //         content: "4".to_string(),
        //     },
        // );
        // assert_eq!(text.to_string(), "2143".to_string());
    }
}
