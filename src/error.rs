use serde::Serialize;

pub mod code;

#[derive(Serialize, Debug)]
pub struct SocketError {
    pub code: u32,
    pub message: String,
}
