use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ContentFragment {
    Text { text: String, bold: bool, italic: bool, underline: bool, strike: bool, color: String, small_font: bool },
    Link { text: String, url: String, small_font: bool },
    Image { url: String, thumb_url: String, size: i64 },
    Quote { html: String, author_and_time: String, tid: String, pid: String },
    Attachment { url: String, name: String, desc: String },
    GoToFloor { text: String, tid: String, pid: String, floor: i32, author: String },
    Email(String), Notice(String), LineBreak,
}

#[derive(Debug, Clone)]
pub struct PostItem {
    pub post_id: String, pub author: String, pub uid: String, pub avatar_url: String,
    pub time_post: String, pub floor: i32, pub page: i32, pub warned: bool,
    pub post_status: String, pub contents: Vec<ContentFragment>,
    pub images: Vec<ImageInfo>, pub poll: Option<PollData>,
}

#[derive(Debug, Clone)]
pub struct ImageInfo { pub url: String, pub thumb_url: String, pub size: i64, pub floor: i32, pub author: String, pub index: usize }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData { pub title: String, pub formhash: String, pub max_answer: i32, pub options: Vec<PollOption>, pub footer: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption { pub option_id: String, pub text: String, pub rates: String }

#[derive(Debug, Clone, Default)]
pub struct ThreadDetail { pub posts: Vec<PostItem>, pub title: String, pub tid: String, pub fid: i32, pub page: i32, pub last_page: i32 }

#[derive(Debug, Clone)]
pub struct PostArg { pub tid: String, pub pid: String, pub fid: i32, pub floor: i32, pub page: i32, pub content: String, pub subject: String, pub typeid: String, pub delete: bool, pub poll_answers: Vec<String> }
impl Default for PostArg { fn default() -> Self { Self { tid: String::new(), pid: String::new(), fid: 0, floor: 0, page: 1, content: String::new(), subject: String::new(), typeid: String::new(), delete: false, poll_answers: vec![] } } }

#[derive(Debug, Clone)]
pub struct PrePostInfo { pub formhash: String, pub text: String, pub quote_text: String, pub uid: String, pub hash: String, pub subject: String, pub deletable: bool, pub notice_author: String, pub notice_author_msg: String, pub notice_trim_str: String, pub images: Vec<String>, pub attaches: Vec<String>, pub type_id: String, pub type_values: Vec<(String, String)> }
