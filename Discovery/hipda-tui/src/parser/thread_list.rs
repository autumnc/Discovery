use scraper::{Html, Selector};
use crate::model::thread::{ThreadItem, ThreadList};
use crate::utils::*;

pub fn parse(html: &str) -> ThreadList {
    let doc = Html::parse_document(html);
    let mut list = ThreadList::default();
    list.parsed = true;
    let tbody_sel = Selector::parse("tbody").unwrap();
    // Parse forum-level max_page from pagination
    if let Ok(sel) = Selector::parse("div.pages_btns div.pages a, div.pages a") {
        for a in doc.select(&sel) {
            if let Ok(n) = a.text().collect::<String>().trim().parse::<i32>() {
                if n > list.max_page { list.max_page = n; }
            }
        }
    }
    if let Ok(sel) = Selector::parse("div.pages_btns div.pages strong, div.pages strong") {
        for s in doc.select(&sel) {
            if let Ok(n) = s.text().collect::<String>().trim().parse::<i32>() {
                if n > list.max_page { list.max_page = n; }
            }
        }
    }
    if list.max_page < 1 { list.max_page = 1; }

    for tbody in doc.select(&tbody_sel) {
        let id = tbody.value().attr("id").unwrap_or("");
        if !id.starts_with("normalthread_") { continue; }
        let mut t = ThreadItem { tid: String::new(), title: String::new(), title_color: String::new(), author: String::new(), author_id: String::new(), avatar_url: String::new(), last_post_author: String::new(), count_cmts: String::new(), count_views: String::new(), time_create: String::new(), time_update: String::new(), with_pic: false, is_new: false, is_poll: false, sticky: false, thread_type: String::new(), max_page: 1 };
        // Redirect threads in Discuz! 7.2 lack the <span> wrapper and don't have tid=
        let span_a = Selector::parse("th.subject span a").unwrap();
        let bare_a = Selector::parse("th.subject a").unwrap();
        if let Some(link) = tbody.select(&span_a).next().or_else(|| tbody.select(&bare_a).next()) {
            t.title = link.text().collect::<String>();
            t.tid = get_middle_string(link.value().attr("href").unwrap_or(""), "tid=", "&");
            if t.tid.is_empty() { continue; }
        } else { continue; }
        if let Some(el) = tbody.select(&Selector::parse("th.subject em a").unwrap()).next() { t.thread_type = el.text().collect(); }
        t.is_poll = tbody.select(&Selector::parse("td.icon img").unwrap()).any(|img| img.value().attr("src").unwrap_or("").contains("/poll"));
        t.sticky = tbody.select(&Selector::parse("td.folder img").unwrap()).any(|img| img.value().attr("src").unwrap_or("").contains("/pin_"));
        let author_sel = Selector::parse("td.author cite a").unwrap();
        if let Some(a) = tbody.select(&author_sel).next() {
            t.author = a.text().collect::<String>();
            let uid = get_middle_string(a.value().attr("href").unwrap_or(""), "uid=", "&");
            t.author_id = uid.clone();
            t.avatar_url = crate::constants::get_forum_by_fid(0).map(|_| String::new()).unwrap_or_default();
        } else { continue; }
        if let Some(el) = tbody.select(&Selector::parse("td.author em").unwrap()).next() { t.time_create = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("td.nums strong").unwrap()).next() { t.count_cmts = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("td.nums em").unwrap()).next() { t.count_views = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("td.lastpost cite").unwrap()).next() { t.last_post_author = el.text().collect(); }
        t.with_pic = tbody.select(&Selector::parse("img.attach").unwrap()).any(|img| img.value().attr("src").unwrap_or("").ends_with("image_s.gif"));
        if let Some(p) = tbody.select(&Selector::parse("span.threadpages a").unwrap()).last() {
            if let Ok(n) = get_middle_string(p.value().attr("href").unwrap_or(""), "page=", "&").parse::<i32>() { t.max_page = t.max_page.max(n); }
        }
        list.threads.push(t);
    }
    list
}
