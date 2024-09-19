use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: Option<String>,
    pub title: String,
    pub author: String,
    pub year: u32,
}
