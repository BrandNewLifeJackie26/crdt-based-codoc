use crate::block::Block;
use std::{error::Error, fmt::Display};

pub type CRDTResult<T> = Result<T, Box<(dyn Error + Send + Sync)>>;

/// basic error types that can occur when running the tribbler service.
#[derive(Debug, Clone)]
pub enum CRDTError {
    /// used when an operation is called for a particular user who does not
    /// exist
    UserDoesNotExist(String),
}

impl Display for CRDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            CRDTError::UserDoesNotExist(x) => format!("user \"{}\" does not exist", x),
            x => format!("{:?}", x),
        };
        write!(f, "{}", x)
    }
}

// general type
pub type ClientID = u32;

pub type Updates = Vec<Block>;

#[derive(Debug, Clone)]
pub struct Peer {
    pub client_id: ClientID,
    pub ip_addr: String,
}

impl Peer {}
