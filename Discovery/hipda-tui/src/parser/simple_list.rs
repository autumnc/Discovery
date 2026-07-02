use scraper::{Html, Selector};
use crate::model::simple::*;
use crate::utils::*;

pub fn parse_search(html: &str) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();

    // Parse max_page from pagination
    if let Ok(sel) = Selector::parse("div.pages_btns div.pages a, div.pages a") {
        for a in doc.select(&sel) {
            if let Ok(n) = a.text().collect::<String>().trim().parse::<i32>() {
                if n > list.max_page { list.max_page = n; }
            }
        }
    }
    // Also check <strong> for current/last page
    if let Ok(sel) = Selector::parse("div.pages_btns div.pages strong, div.pages strong") {
        for s in doc.select(&sel) {
            if let Ok(n) = s.text().collect::<String>().trim().parse::<i32>() {
                if n > list.max_page { list.max_page = n; }
            }
        }
    }
    if list.max_page < 1 { list.max_page = 1; }

    // Extract searchid for pagination
    if let Ok(sel) = Selector::parse("div.pages_btns div.pages a, div.pages a") {
        if let Some(a) = doc.select(&sel).next() {
            list.search_id = get_middle_string(a.value().attr("href").unwrap_or(""), "searchid=", "&");
        }
    }

    // Method 1: tbody-based parsing (search results, thread lists)
    for tbody in doc.select(&Selector::parse("tbody").unwrap()) {
        let mut item = SimpleListItem::default();
        let mut found = false;
        for sel_str in &["tr th.subject a", "tr th a"] {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(link) = tbody.select(&sel).next() {
                    item.tid = get_middle_string(link.value().attr("href").unwrap_or(""), "tid=", "&");
                    if !item.tid.is_empty() {
                        item.title = link.text().collect();
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found { continue; }
        if let Some(el) = tbody.select(&Selector::parse("tr td.author cite a").unwrap()).next() {
            item.author = el.text().collect();
            item.uid = get_middle_string(el.value().attr("href").unwrap_or(""), "uid=", "&");
        }
        if let Some(el) = tbody.select(&Selector::parse("tr td.author em").unwrap()).next() { item.time = el.text().collect(); }
        if let Some(el) = tbody.select(&Selector::parse("tr td.forum").unwrap()).next() { item.forum = el.text().collect(); }
        list.items.push(item);
    }

    list
}

/// Parse my.php?item=threads (MyPosts) — table.datatable with viewthread.php links
pub fn parse_my_posts(html: &str) -> SimpleList {
    parse_datatable(html, "viewthread.php?tid=", false)
}

/// Parse my.php?item=posts (MyReplies) — table.datatable with redirect links, alternating rows
pub fn parse_my_replies(html: &str) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();
    parse_page_nav(&doc, &mut list);

    let table = match doc.select(&Selector::parse("table.datatable").unwrap()).next() {
        Some(t) => t,
        None => return list,
    };
    let trs: Vec<_> = table.select(&Selector::parse("tr").unwrap()).collect();
    for i in (1..trs.len()).step_by(2) {
        // odd rows have title (1-indexed, skip header row)
        if let Ok(th_sel) = Selector::parse("th") {
            if let Some(th) = trs[i].select(&th_sel).next() {
                if let Some(link) = th.select(&Selector::parse("a").unwrap()).next() {
                    let href = link.value().attr("href").unwrap_or("");
                    let tid = get_middle_string(href, "ptid=", "&");
                    if tid.is_empty() { continue; }
                    let mut item = SimpleListItem::default();
                    item.tid = tid;
                    item.pid = get_middle_string(href, "pid=", "&");
                    item.title = link.text().collect();
                    if let Some(lp) = trs[i].select(&Selector::parse("td.lastpost").unwrap()).next() {
                        item.time = lp.text().collect();
                    }
                    if let Some(f) = trs[i].select(&Selector::parse("td.forum").unwrap()).next() {
                        item.forum = f.text().collect();
                    }
                    // odd+1 row has reply snippet (info)
                    if i + 1 < trs.len() {
                        if let Some(th) = trs[i+1].select(&Selector::parse("th").unwrap()).next() {
                            item.info = th.text().collect();
                        }
                    }
                    list.items.push(item);
                }
            }
        }
    }
    list
}

/// Parse my.php?item=favorites or buddylist — table.datatable tbody tr
pub fn parse_favorites(html: &str) -> SimpleList {
    parse_datatable(html, "tid=", true)
}

/// Parse pm.php — ul.pm_list with li items
pub fn parse_sms(html: &str) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();
    let ul = match doc.select(&Selector::parse("ul.pm_list").unwrap()).next() {
        Some(u) => u,
        None => return list,
    };
    for li in ul.select(&Selector::parse("li").unwrap()) {
        let mut item = SimpleListItem::default();
        // author from p.cite > cite > a
        if let Some(cite) = li.select(&Selector::parse("p.cite cite").unwrap()).next() {
            if let Some(a) = cite.select(&Selector::parse("a").unwrap()).next() {
                item.author = a.text().collect();
                item.uid = get_middle_string(a.value().attr("href").unwrap_or(""), "uid=", "&");
            }
            // time is ownText of p.cite
            item.time = cite.text().collect::<Vec<_>>().join("");
            // Remove the author text from the collected text
            if let Some(pos) = item.time.find(&item.author) {
                item.time = item.time[pos + item.author.len()..].trim().to_string();
            }
        }
        // summary as title
        if let Some(summary) = li.select(&Selector::parse("div.summary").unwrap()).next() {
            item.title = summary.text().collect();
        }
        // check for new PM indicator
        if let Some(img) = li.select(&Selector::parse("p.cite img").unwrap()).next() {
            if img.value().attr("src").unwrap_or("").contains("pm_new") { item.is_new = true; }
        }
        if !item.author.is_empty() { list.items.push(item); }
    }
    list
}

/// Parse notice.php — ul.feed with li > div.f_*
pub fn parse_notify(html: &str) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();
    let feed = match doc.select(&Selector::parse("ul.feed").unwrap()).next() {
        Some(f) => f,
        None => return list,
    };
    for li in feed.select(&Selector::parse("li").unwrap()) {
        if let Some(div) = li.select(&Selector::parse("div").unwrap()).next() {
            let cls = div.value().attr("class").unwrap_or("");
            let mut item = SimpleListItem::default();
            if cls.contains("f_thread") {
                // user replied to your thread
                for a in div.select(&Selector::parse("a").unwrap()) {
                    let href = a.value().attr("href").unwrap_or("");
                    if href.contains("redirect.php") {
                        item.title = a.text().collect();
                        item.tid = get_middle_string(href, "ptid=", "&");
                        item.pid = get_middle_string(href, "pid=", "&");
                    } else if href.contains("space.php") {
                        item.info += &(a.text().collect::<String>() + " ");
                    }
                }
                if let Some(em) = div.select(&Selector::parse("em").unwrap()).next() {
                    item.time = em.text().collect();
                }
                item.info += "回复了您的帖子";
                item.is_new = true;
            } else if cls.contains("f_quote") || cls.contains("f_reply") {
                for a in div.select(&Selector::parse("a").unwrap()) {
                    let href = a.value().attr("href").unwrap_or("");
                    if href.contains("space.php") {
                        item.author = a.text().collect();
                        item.uid = get_middle_string(href, "uid=", "&");
                    } else if href.contains("viewthread.php") || href.contains("redirect.php") {
                        if item.title.is_empty() { item.title = a.text().collect(); }
                        if href.contains("redirect.php") {
                            item.tid = get_middle_string(href, "ptid=", "&");
                            item.pid = get_middle_string(href, "pid=", "&");
                        } else {
                            item.tid = get_middle_string(href, "tid=", "&");
                        }
                    }
                }
                if let Some(em) = div.select(&Selector::parse("em").unwrap()).next() {
                    item.time = em.text().collect();
                }
                item.is_new = !div.select(&Selector::parse("img").unwrap()).next()
                    .map(|i| i.value().attr("src").unwrap_or("")).unwrap_or("").is_empty();
                if let Some(sum) = div.select(&Selector::parse(".summary").unwrap()).next() {
                    item.info = sum.text().collect();
                }
            } else if cls.contains("f_manage") {
                item.title = "系统信息".to_string();
                if let Some(a) = div.select(&Selector::parse("a").unwrap()).next() {
                    item.tid = get_middle_string(a.value().attr("href").unwrap_or(""), "tid=", "&");
                }
                item.info = div.text().collect();
            } else if cls.contains("f_buddy") {
                item.title = "好友信息".to_string();
                if let Some(a) = div.select(&Selector::parse("a").unwrap()).next() {
                    item.author = a.text().collect();
                    item.uid = get_middle_string(a.value().attr("href").unwrap_or(""), "uid=", "&");
                }
                item.info = div.text().collect();
            }
            if item.tid.is_empty() && item.author.is_empty() && item.info.is_empty() { continue; }
            list.items.push(item);
        }
    }
    list
}

/// Shared parser for table.datatable pages (MyPosts, Favorites, Attention)
fn parse_datatable(html: &str, href_key: &str, use_tbody: bool) -> SimpleList {
    let doc = Html::parse_document(html);
    let mut list = SimpleList::default();
    parse_page_nav(&doc, &mut list);

    let row_sel = if use_tbody { "table.datatable tbody tr" } else { "table.datatable tr" };
    let trs = match Selector::parse(row_sel) {
        Ok(sel) => doc.select(&sel).collect::<Vec<_>>(),
        Err(_) => return list,
    };
    for tr in trs {
        let mut item = SimpleListItem::default();
        if let Ok(th_sel) = Selector::parse("th") {
            if let Some(th) = tr.select(&th_sel).next() {
                if let Some(a) = th.select(&Selector::parse("a").unwrap()).next() {
                    let href = a.value().attr("href").unwrap_or("");
                    item.tid = get_middle_string(href, href_key, "&");
                    if item.tid.is_empty() { continue; }
                    item.title = a.text().collect();
                } else {
                    continue;
                }
            } else { continue; }
        }
        if let Some(lp) = tr.select(&Selector::parse("td.lastpost").unwrap()).next() {
            item.time = lp.text().collect::<Vec<_>>().concat().trim().to_string();
        }
        if let Some(f) = tr.select(&Selector::parse("td.forum").unwrap()).next() {
            item.forum = f.text().collect::<Vec<_>>().concat().trim().to_string();
        }
        list.items.push(item);
    }
    list
}

fn parse_page_nav(doc: &scraper::Html, list: &mut SimpleList) {
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
}
