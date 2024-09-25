use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Book {
    pub(crate) id: Option<String>,
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) year: u32,
}
