use scraper::{Html, Selector};
use crate::model::user::UserInfo;
use crate::utils::*;
pub fn parse(html: &str) -> Option<UserInfo> {
    let doc = Html::parse_document(html);
    let mut info = UserInfo::default();
    if let Some(el) = doc.select(&Selector::parse("div#profilecontent div.itemtitle h1").unwrap()).next() { info.username = el.text().collect::<String>().trim().to_string(); }
    if let Some(el) = doc.select(&Selector::parse("div#profilecontent div.itemtitle ul li").unwrap()).next() { info.uid = get_middle_string(&el.text().collect::<String>(), "(UID:", ")").trim().to_string(); }
    Some(info)
}
