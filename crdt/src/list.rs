// use std::collections::{LinkedList};
// use rand::Rng;
// // use intrusive_collections::linked_list::{CursorMut, LinkedList};

// pub const ID_START: u128 = u128::MIN;
// pub const ID_END: u128 = u128::MAX;

// #[derive(Clone)]
// pub struct UID {
//     // id_list: Vec<i32>,
//     pub id: u64,
// }

// // TODO: need to implement?
// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
// pub struct ItemID {
//     pub id: u128,
// }

// impl ItemID {
//     pub fn generate_id(left_id: ItemID, right_id: ItemID) -> ItemID {
//         // Find the first number that is different in two list
//         // let mut i = 0;
//         // loop {
//         //     if i < left_id.id.len() && i < right_id.id.len() && left_id.id[i] == right_id.id[i] {
//         //         i += 1;
//         //         continue;
//         //     }
//         //     break;
//         // }

//         // [0,2,3] v.s. [0,2,5]  => [0,2,4]
//         // [0,2] v.s. [0,2,5] => [0,2,4], [0,0] v.s. [0,0,0] => [0,0,0,MAX]
//         // [0,2,3,4] v.s. [0,2,5] => [0,2,4,4]
//         // [0,2] v.s. [0,2] => [0,2,1]
//         // let new_item_id = left_id.clone();
//         // if i >= left_id.id.len() {
//         //     if i >= right_id.id.len() {
//         //         new_item_id.id.push(1);
//         //     } else {
//         //         new_item_id.id.push(right_id.id[i] - 1);
//         //         if new_item_id.id[i] == 0 {
//         //             new_item_id.id[i] =
//         //         }
//         //     }
//         // }

//         // new_item_id

//         let mut rng = rand::thread_rng();
//         let rand: f64 = 0.2 * rng.gen::<f64>() + 0.9; // random range: [0.9, 1.1) // TODO: can be changed to avoid collision
//         let offset: f64 = (((right_id.id - left_id.id) / 2) as f64) * rand;
//         ItemID {id: left_id.id + offset as u128}
//     }
// }

// #[derive(Clone)]
// pub struct Content {
//     pub content: String,
// }

// #[derive(Clone)]
// pub struct Item {
//     pub uid: UID,
//     pub item_id: ItemID,
//     pub clock: u64,
//     pub origin_left: ItemID,
//     pub origin_right: ItemID,
//     pub is_deleted: bool,
//     pub content: Content,
// }

// // Use YATA algorithm to represent a CRDT list
// pub struct List {
//     pub items: LinkedList<Item>,
// }

// impl List {
//     pub fn new(uid: UID) -> List {
//         let start_item = Item {
//             uid: uid.clone(),
//             item_id: ItemID { id: ID_START },
//             clock: 0,
//             origin_left: ItemID { id: ID_START },
//             origin_right: ItemID { id: ID_START },
//             is_deleted: false,
//             content: Content {
//                 content: "".to_string(),
//             },
//         };
//         let end_item = Item {
//             uid: uid.clone(),
//             item_id: ItemID { id: ID_END },
//             clock: 0,
//             origin_left: ItemID { id: ID_END },
//             origin_right: ItemID { id: ID_END },
//             is_deleted: false,
//             content: Content {
//                 content: "".to_string(),
//             },
//         };

//         let items = LinkedList::from([start_item, end_item]);
//         List { items }
//     }

//     // Local insert:
//     // Insert the content into a specific position into the current List
//     // List will only be modified by the current insertion
//     // Returns the newly created ItemID
//     pub fn insert(&self, left_id: ItemID, right_id: ItemID, content: Content) -> ItemID {
//         assert!(left_id < right_id, "left_id ({}) should be less than right_id ({})", left_id.id, right_id.id);

//         // Start from the second item, find the item left to inserted content
//         // TODO: keep a mapping from ItemID to Item inside the linkedlist to support faster insertion
//         // TODO: use intrusive-collections to insert element into a linkedlist
//         // let iter = self.items.iter_mut();
//         // iter.next();
//         // while let Some(item) = iter.next() {
//         //     if item.item_id == left_id {
//         //         break;
//         //     }
//         // }

//         // ItemID { id: 0 }
//         todo!()
//     }

//     // TODO: reserve insert(&self, ins_pos: u64, content: Content)
//     // if we cannot get item_id of left and right items
//     pub fn delete(&self, deleted_id: ItemID) {}

//     pub fn to_string(&self) -> String {
//         let end_item_id = ItemID { id: ID_END };
//         let mut res: Vec<String> = vec![];

//         // Start from the second Item
//         let mut iter = self.items.iter();
//         iter.next();
//         while let Some(item) = iter.next() {
//             if item.item_id == end_item_id {
//                 break
//             }
//             res.push(item.content.content.clone());
//         }
//         res.into_iter().collect()
//     }
// }
