use scraper::{Html, Selector};
use crate::model::simple::*;
use crate::utils::*;

pub fn parse_search(html: &str) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();
    for tbody in doc.select(&Selector::parse("tbody").unwrap()) {
        let mut item = SimpleListItem { tid: String::new(), pid: String::new(), uid: String::new(), title: String::new(), author: String::new(), avatar_url: String::new(), time: String::new(), forum: String::new(), info: String::new(), is_new: false };
        if let Some(link) = tbody.select(&Selector::parse("tr th.subject a").unwrap()).next() {
            item.tid = get_middle_string(link.value().attr("href").unwrap_or(""), "tid=", "&");
            item.title = link.text().collect();
        } else { continue; }
        if let Some(el) = tbody.select(&Selector::parse("tr td.author cite a").unwrap()).next() { item.author = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("tr td.author em").unwrap()).next() { item.time = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("tr td.forum").unwrap()).next() { item.forum = el.text().collect(); }
        list.items.push(item);
    }
    list
}
pub fn parse_sms(html: &str) -> SimpleList { parse_search(html) }
pub fn parse_sms_detail(html: &str) -> SimpleList { parse_search(html) }
pub fn parse_notify(html: &str) -> SimpleList { parse_search(html) }
pub fn parse_favorites(html: &str) -> SimpleList { parse_search(html) }
pub fn parse_my_reply(html: &str) -> SimpleList { parse_search(html) }
