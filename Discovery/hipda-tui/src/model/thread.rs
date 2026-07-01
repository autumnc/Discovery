use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadItem {
    pub tid: String, pub title: String, pub title_color: String,
    pub author: String, pub author_id: String, pub avatar_url: String,
    pub last_post_author: String, pub count_cmts: String, pub count_views: String,
    pub time_create: String, pub time_update: String,
    pub with_pic: bool, pub is_new: bool, pub is_poll: bool, pub sticky: bool,
    pub thread_type: String, pub max_page: i32,
}
#[derive(Debug, Clone, Default)]
pub struct ThreadList { pub threads: Vec<ThreadItem>, pub uid: String, pub parsed: bool }
