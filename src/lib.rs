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
    use crate::block::{Block, BlockID, Content};
    use crate::block_store;
    use crate::doc::Doc;
    use crate::list::*;
    use crate::list::*;
    use crate::utils::ClientID;

    // Local insert to a single doc (one letter at a time),
    // there is no need to use transaction if no sync is needed
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_insert_single() {
        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        doc.insert_local(
            Content {
                content: "1".to_string(),
            },
            0,
        )
        .await;
        assert_eq!(doc.to_string().await, "1".to_string());

        doc.insert_local(
            Content {
                content: "2".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "12".to_string());

        // Insert pos is larger than length
        doc.insert_local(
            Content {
                content: "3".to_string(),
            },
            10,
        )
        .await;
        assert_eq!(doc.to_string().await, "123".to_string());

        doc.insert_local(
            Content {
                content: "4".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "1423".to_string());
    }

    // Local insert to a single doc, (may insert more than one letter at a time)
    // there is no need to use transaction if no sync is needed
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn local_insert_multiple() {
        let cid = 1 as ClientID;
        let mut doc = Doc::new("text".to_string(), cid);

        doc.insert_local(
            Content {
                content: "123".to_string(),
            },
            0,
        )
        .await;
        assert_eq!(doc.to_string().await, "123".to_string());

        doc.insert_local(
            Content {
                content: "45".to_string(),
            },
            1,
        )
        .await;
        assert_eq!(doc.to_string().await, "14523".to_string());

        // Insert pos is larger than length
        doc.insert_local(
            Content {
                content: "6".to_string(),
            },
            10,
        )
        .await;
        assert_eq!(doc.to_string().await, "145236".to_string());

        doc.insert_local(
            Content {
                content: "789".to_string(),
            },
            4,
        )
        .await;
        assert_eq!(doc.to_string().await, "145278936".to_string());
    }
}
