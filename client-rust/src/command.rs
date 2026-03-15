use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandItem {
    pub index: u32,
    pub instruction: String,
    pub tag: String,
}
