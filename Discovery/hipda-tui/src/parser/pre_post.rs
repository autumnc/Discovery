use scraper::{Html, Selector};
use crate::model::post::PrePostInfo;

pub fn parse(html: &str) -> Option<PrePostInfo> {
    let doc = Html::parse_document(html);
    let mut info = PrePostInfo { formhash: String::new(), text: String::new(), quote_text: String::new(), uid: String::new(), hash: String::new(), subject: String::new(), deletable: false, notice_author: String::new(), notice_author_msg: String::new(), notice_trim_str: String::new(), images: vec![], attaches: vec![], type_id: String::new(), type_values: vec![] };
    let fh_sel = Selector::parse("input[name=formhash]").unwrap();
    info.formhash = doc.select(&fh_sel).next().and_then(|el| el.value().attr("value")).unwrap_or("").to_string();
    if info.formhash.is_empty() { return None; }
    let subj_sel = Selector::parse("input[name=subject]").unwrap();
    if let Some(el) = doc.select(&subj_sel).next() { info.subject = el.value().attr("value").unwrap_or("").to_string(); }
    let del_sel = Selector::parse("input#delete").unwrap();
    info.deletable = doc.select(&del_sel).next().is_some();
    Some(info)
}
