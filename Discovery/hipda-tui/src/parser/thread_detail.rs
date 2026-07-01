use scraper::{Html, Selector, ElementRef};
use ego_tree::NodeRef;
use crate::model::post::*;
use crate::utils::*;

pub fn parse(html: &str, tid: &str) -> Option<ThreadDetail> {
    let doc = Html::parse_document(html);
    let mut detail = ThreadDetail::default();
    let pages_sel = Selector::parse("div.pages").unwrap();
    if let Some(pages_el) = doc.select(&pages_sel).next() {
        for child in pages_el.children() {
            if let Some(el_ref) = ElementRef::wrap(child) {
                let n = get_int_from_string(&el_ref.text().collect::<Vec<_>>().join(""));
                if n > detail.last_page { detail.last_page = n; }
                if el_ref.value().name() == "strong" { detail.page = n; }
            }
        }
    }
    if !tid.is_empty() { detail.tid = tid.to_string(); }
    let nav_sel = Selector::parse("div#nav").unwrap();
    if let Some(nav) = doc.select(&nav_sel).next() {
        for link in nav.select(&Selector::parse("a").unwrap()) {
            let href = link.value().attr("href").unwrap_or("");
            if href.contains("fid=") { detail.fid = parse_int(&get_middle_string(href, "fid=", "&")); break; }
        }
        detail.title = nav.text().collect::<String>().replace('»', "").trim().to_string();
    }
    let postlist_sel = Selector::parse("div#wrap div#postlist").unwrap();
    let posts_el = doc.select(&postlist_sel).next()?;
    for child in posts_el.children() {
        let Some(post_elem) = ElementRef::wrap(child) else { continue };
        let mut post = PostItem { post_id: String::new(), author: String::new(), uid: String::new(), avatar_url: String::new(), time_post: String::new(), floor: 0, page: detail.page, warned: false, post_status: String::new(), contents: vec![], images: vec![], poll: None };
        let id = post_elem.value().attr("id").unwrap_or("");
        if let Some(pid) = id.strip_prefix("post_") { post.post_id = pid.to_string(); } else { continue; }
        if let Some(el) = post_elem.select(&Selector::parse("div.postinfo div.posterinfo div.authorinfo em").unwrap()).next() {
            let time = el.text().collect::<String>();
            post.time_post = time.chars().skip(4).collect::<String>();
            if post.time_post.is_empty() { post.time_post = time; }
        }
        if let Some(el) = post_elem.select(&Selector::parse("div.postinfo strong a em").unwrap()).next() { post.floor = parse_int(&el.text().collect::<String>()); }
        if let Some(el) = post_elem.select(&Selector::parse("td.postauthor div.postinfo a").unwrap()).next() {
            post.uid = get_middle_string(el.value().attr("href").unwrap_or(""), "uid=", "&");
            post.author = el.text().collect::<String>();
        } else { continue; }
        // Parse content from t_msgfont
        let content_sel = Selector::parse("td.postcontent div.defaultpost div.postmessage div.t_msgfontfix table tbody tr td.t_msgfont").unwrap();
        if let Some(content_el) = post_elem.select(&content_sel).next() {
            for child in content_el.children() { walk_node(child, &mut post, 1, &mut Vec::new()); }
        }
        // Attached images
        for dl in post_elem.select(&Selector::parse("dl.attachimg").unwrap()) {
            let mut size: i64 = 0;
            if let Some(em) = dl.select(&Selector::parse("em").unwrap()).next() {
                size = parse_size_text(&get_middle_string(&em.text().collect::<String>(), "(", ")"));
            }
            if let Some(img) = dl.select(&Selector::parse("img").unwrap()).next() {
                let src = img.value().attr("src").unwrap_or("");
                let file = img.value().attr("file").unwrap_or("");
                let onclick = img.value().attr("onclick").unwrap_or("");
                let url = if !onclick.is_empty() { onclick } else { file };
                let thumb = if src.contains("thumb.") { src } else { "" };
                let full = if url.is_empty() { thumb } else { url };
                post.contents.push(ContentFragment::Image { url: abs_url(full), thumb_url: abs_url(thumb), size });
            }
        }
        detail.posts.push(post);
    }
    Some(detail)
}

fn walk_node<'a>(node: NodeRef<'a, scraper::Node>, post: &mut PostItem, _level: usize, styles: &mut Vec<TextStyleState>) {
    match node.value() {
        scraper::node::Node::Text(t) => {
            let text = t.text.trim().to_string();
            if text.is_empty() { return; }
            let s = styles.last().cloned().unwrap_or_default();
            post.contents.push(ContentFragment::Text { text, bold: s.bold, italic: s.italic, underline: s.underline, strike: s.strike, color: s.color, small_font: s.small_font });
        }
        scraper::node::Node::Element(el) => {
            match el.name() {
                "br" => { post.contents.push(ContentFragment::LineBreak); }
                "img" => {
                    if let Some(elem) = ElementRef::wrap(node) {
                        let src = elem.value().attr("src").unwrap_or("");
                        if src.contains("://") && !src.contains("data:image/") && !src.contains("smilies") && !src.contains("common") {
                            post.contents.push(ContentFragment::Image { url: abs_url(src), thumb_url: String::new(), size: 0 });
                        }
                    }
                }
                "a" => {
                    if let Some(elem) = ElementRef::wrap(node) {
                        let url = elem.value().attr("href").unwrap_or("");
                        let text: String = elem.text().collect();
                        if !url.starts_with("javascript:") && !url.is_empty() {
                            post.contents.push(ContentFragment::Link { text, url: url.to_string(), small_font: false });
                        }
                        for child in elem.children() { walk_node(child, post, _level + 1, styles); }
                    }
                }
                "div" => {
                    if let Some(elem) = ElementRef::wrap(node) {
                        if elem.value().attr("class") == Some("quote") {
                            post.contents.push(ContentFragment::Quote { html: elem.inner_html(), author_and_time: elem.text().collect(), tid: String::new(), pid: String::new() });
                            return;
                        }
                    }
                    for child in node.children() { walk_node(child, post, _level + 1, styles); }
                }
                _ => { for child in node.children() { walk_node(child, post, _level + 1, styles); } }
            }
        }
        _ => { for child in node.children() { walk_node(child, post, _level + 1, styles); } }
    }
}

#[derive(Clone, Default)]
struct TextStyleState { bold: bool, italic: bool, underline: bool, strike: bool, color: String, small_font: bool }

fn abs_url(url: &str) -> String { if url.contains("://") || url.is_empty() { url.to_string() } else { format!("{}{}", *crate::constants::BASE_URL, url) } }
