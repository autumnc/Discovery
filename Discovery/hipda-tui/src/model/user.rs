use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserInfo { pub username: String, pub uid: String, pub avatar_url: String, pub online: bool, pub formhash: String, pub detail: String }
