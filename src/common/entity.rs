use axum::extract::ws;
use serde::{Deserialize, Serialize};
use std::result::Result;
use tokio_tungstenite::tungstenite;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TransferRequest {
    pub file_hash: String,
    pub file_size: usize,
    pub file_name: String,
    pub file_dir: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TransferResponse {
    pub file_hash: String,
    pub sync_size: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DeleteRequest {
    pub file_hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TransferControlMessage {
    Request(TransferRequest),
    Response(TransferResponse),
    Delete(TransferResponse),
    Error(String),
}

impl From<TransferControlMessage> for ws::Message {
    fn from(val: TransferControlMessage) -> Self {
        ws::Message::Text(serde_json::to_string(&val).unwrap())
    }
}

impl From<TransferControlMessage> for tungstenite::Message {
    fn from(val: TransferControlMessage) -> Self {
        tungstenite::Message::Text(serde_json::to_string(&val).unwrap())
    }
}

impl TryFrom<&str> for TransferControlMessage {
    type Error = ();
    fn try_from(text: &str) -> Result<Self, ()> {
        Ok(serde_json::from_str::<TransferControlMessage>(text).map_err(|e| ())?)
    }
}
