use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleListItem { pub tid: String, pub pid: String, pub uid: String, pub title: String, pub author: String, pub avatar_url: String, pub time: String, pub forum: String, pub info: String, pub is_new: bool }
#[derive(Debug, Clone, Default)]
pub struct SimpleList { pub items: Vec<SimpleListItem>, pub max_page: i32, pub search_id: String }
