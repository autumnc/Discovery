use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forum { pub id: i32, pub name: String }
#[derive(Debug, Clone)]
pub struct ForumGroup { pub forums: Vec<Forum> }
