use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use crate::config::Config;
use crate::http::HttpClient;
use crate::model::post::ThreadDetail;
use crate::model::simple::SimpleList;
use crate::model::thread::ThreadList;
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq)]
enum LoginField { Username, Password }

async fn do_login(http: &HttpClient, username: &str, password: &str) -> Result<String, String> {
    let login_page = http.get(&crate::constants::LOGIN_GET_FORMHASH).await
        .map_err(|e| format!("网络错误: {}", e))?;
    let formhash = {
        let doc = scraper::Html::parse_document(&login_page);
        let sel = scraper::Selector::parse("input[name=formhash]").unwrap();
        doc.select(&sel).next().and_then(|el| el.value().attr("value")).unwrap_or("58734250").to_string()
    };
    let mut params = HashMap::new();
    params.insert("formhash".into(), formhash);
    params.insert("loginfield".into(), "username".into());
    params.insert("username".into(), username.to_string());
    params.insert("password".into(), password.to_string());
    params.insert("questionid".into(), "0".into());
    params.insert("answer".into(), String::new());
    params.insert("loginsubmit".into(), "true".into());
    params.insert("referer".into(), "index.php".into());
    let rsp = http.post(&crate::constants::LOGIN_SUBMIT, &params).await
        .map_err(|e| format!("登录失败: {}", e))?;
    if rsp.contains("欢迎您回来") { Ok(rsp) } else { Err("用户名或密码错误".into()) }
}

#[derive(Clone)]
enum AppAction {
    LoginResult(Result<String, String>),
    ThreadListResult(Result<ThreadList, String>),
    ThreadDetailResult(Result<ThreadDetail, String>, String),
    ImageData(Vec<Vec<u8>>),
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let http = Arc::new(HttpClient::new()?);
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<AppAction>();
    let prefilled_username = config.username.clone();
    let prefilled_password = config.password.clone();
    let has_creds = !prefilled_username.is_empty() && !prefilled_password.is_empty();

    let mut app = App {
        http: http.clone(), config,
        screen: if has_creds { Screen::Main } else { Screen::Login },
        login_username: prefilled_username.clone(),
        login_password: prefilled_password.clone(),
        login_focus: LoginField::Username, login_error: String::new(), login_loading: false,
        forums: vec![], selected_forum: 0,
        threads: ThreadList::default(), selected_thread: 0, thread_page: 1, thread_total_pages: 1,
        detail: None, detail_scroll: 0, detail_page: 1,
        simple_list: SimpleList::default(), simple_list_type: SimpleListType::Search, simple_selected: 0,
        sms_detail_list: SimpleList::default(), sms_selected: 0, sms_uid: String::new(), sms_username: String::new(),
        composing: false, compose_content: String::new(), compose_subject: String::new(), compose_cursor: 0,
        compose_mode: PostMode::ReplyThread,
        reply_tid: String::new(), reply_pid: String::new(), reply_fid: 0,
        search_query: String::new(), search_mode: false,
        status_msg: String::new(), status_error: false, loading: false,
        tabs: vec!["Forums", "MyPosts", "MyReplies", "Favorites", "Attention", "SMS", "Notify", "Search"],
        active_tab: 0,
        action_tx: action_tx.clone(),
        show_forum_panel: false, detail_folded: false, show_list_detail: false,
        list_scroll: Cell::new(0),
        show_images: false, image_data: vec![], image_index: 0,
        img_area: Cell::new(None),
    };
    app.load_forums();

    if has_creds {
        app.login_loading = true;
        let http = http.clone();
        let tx = action_tx.clone();
        let u = prefilled_username;
        let p = prefilled_password;
        tokio::spawn(async move { let _ = tx.send(AppAction::LoginResult(do_login(&http, &u, &p).await)); });
    }

    loop {
        terminal.draw(|f| app.render(f))?;
        if app.show_images && !app.image_data.is_empty() {
            if let Some((col, row)) = app.img_area.get() {
                let idx = app.image_index.min(app.image_data.len().saturating_sub(1));
                let mut stdout = std::io::stdout();
                let _ = execute!(stdout, crossterm::cursor::MoveTo(col, row));
                let _ = stdout.write_all(&app.image_data[idx]);
                let _ = stdout.flush();
            }
        }
        while let Ok(action) = action_rx.try_recv() { app.handle_action(action); }
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                match app.screen {
                    Screen::Login => app.handle_login_key(key.code),
                    Screen::Main => app.handle_main_key(key.code),
                    Screen::Quit => {}
                }
            }
        }
        if app.screen == Screen::Quit { break; }
    }
    ratatui::restore();
    Ok(())
}

#[derive(Clone, Copy, PartialEq)] enum Screen { Login, Main, Quit }
#[derive(Clone, Copy, PartialEq)] pub enum SimpleListType { Search, MyPosts, MyReplies, Favorites, Attention, Sms, Notify, SmsDetail }
#[derive(Clone, Copy, PartialEq)] pub enum PostMode { ReplyThread, ReplyPost, QuotePost, NewThread, EditPost }

struct App {
    http: Arc<HttpClient>, config: Config, screen: Screen,
    login_username: String, login_password: String, login_focus: LoginField, login_error: String, login_loading: bool,
    forums: Vec<(i32, String)>, selected_forum: usize,
    threads: ThreadList, selected_thread: usize, thread_page: i32, thread_total_pages: i32,
    detail: Option<ThreadDetail>, detail_scroll: usize, detail_page: i32,
    simple_list: SimpleList, simple_list_type: SimpleListType, simple_selected: usize,
    sms_detail_list: SimpleList, sms_selected: usize, sms_uid: String, sms_username: String,
    composing: bool, compose_content: String, compose_subject: String, compose_cursor: usize,
    compose_mode: PostMode, reply_tid: String, reply_pid: String, reply_fid: i32,
    search_query: String, search_mode: bool,
    status_msg: String, status_error: bool, loading: bool,
    tabs: Vec<&'static str>, active_tab: usize,
    action_tx: tokio::sync::mpsc::UnboundedSender<AppAction>,
    show_forum_panel: bool, detail_folded: bool, show_list_detail: bool,
    list_scroll: Cell<usize>,
    show_images: bool, image_data: Vec<Vec<u8>>, image_index: usize,
    img_area: Cell<Option<(u16, u16)>>,
}

impl App {
    fn load_forums(&mut self) { self.forums = crate::constants::FORUMS.iter().filter(|f| self.config.forums.contains(&f.id)).map(|f| (f.id, f.name.to_string())).collect(); }
    fn set_status(&mut self, msg: &str, error: bool) { self.status_msg = msg.to_string(); self.status_error = error; }

    fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::LoginResult(Ok(_)) => {
                self.screen = Screen::Main; self.login_loading = false;
                self.set_status("登录成功! 正在加载...", false); self.loading = true;
                self.spawn_load_threads();
            }
            AppAction::LoginResult(Err(msg)) => { self.login_loading = false; self.login_error = msg; }
            AppAction::ThreadListResult(Ok(list)) => {
                self.threads = list; self.thread_total_pages = 1; self.loading = false;
                self.selected_thread = 0; self.list_scroll.set(0);
                let hint = if self.show_list_detail { "d=简洁" } else { "d=详情" };
                self.set_status(&format!("{} 主题 | k/j=上下 Enter=查看 r=刷新 {} b=板块 q=退出", self.threads.threads.len(), hint), false);
            }
            AppAction::ThreadListResult(Err(msg)) => { self.loading = false; self.set_status(&format!("加载失败: {}", msg), true); }
            AppAction::ThreadDetailResult(Ok(detail), tid) => {
                self.loading = false; self.reply_tid = tid; self.detail_page = detail.page;
                self.detail = Some(detail); self.detail_scroll = 0;
                self.set_status("k/j 移动  f 折叠  r 回复  p 图片  Esc 返回", false);
            }
            AppAction::ThreadDetailResult(Err(msg), _) => { self.loading = false; self.set_status(&format!("加载失败: {}", msg), true); }
            AppAction::ImageData(data) => {
                let total = data.len();
                if total > 0 {
                    self.image_data = data;
                    self.image_index = 0;
                    self.set_status(&format!("图片 1/{} | Tab 下一张  p 关闭", total), false);
                } else {
                    self.image_data.clear();
                    self.show_images = false;
                    self.set_status("无图片或下载失败 | k/j 移动  f 折叠  r 回复", false);
                }
            }
        }
    }

    fn spawn_load_threads(&self) {
        if self.forums.is_empty() { return; }
        let http = self.http.clone(); let tx = self.action_tx.clone();
        let fid = self.forums[self.selected_forum].0; let page = self.thread_page;
        tokio::spawn(async move {
            let url = format!("{}forumdisplay.php?fid={}&page={}", *crate::constants::BASE_URL, fid, page);
            match http.get(&url).await {
                Ok(html) => { let _ = tx.send(AppAction::ThreadListResult(Ok(crate::parser::thread_list::parse(&html)))); }
                Err(e) => { let _ = tx.send(AppAction::ThreadListResult(Err(e.to_string()))); }
            }
        });
    }

    fn spawn_load_thread_detail(&self, tid: &str, page: i32) {
        let http = self.http.clone(); let tx = self.action_tx.clone(); let tid = tid.to_string();
        tokio::spawn(async move {
            let url = format!("{}viewthread.php?tid={}&page={}", *crate::constants::BASE_URL, tid, page);
            match http.get(&url).await {
                Ok(html) => {
                    let detail = crate::parser::thread_detail::parse(&html, &tid);
                    match detail {
                        Some(d) => { let _ = tx.send(AppAction::ThreadDetailResult(Ok(d), tid)); }
                        None => { let _ = tx.send(AppAction::ThreadDetailResult(Err("解析失败".into()), tid)); }
                    }
                }
                Err(e) => { let _ = tx.send(AppAction::ThreadDetailResult(Err(e.to_string()), tid)); }
            }
        });
    }

    fn spawn_load_images(&self) {
        let Some(ref detail) = self.detail else { return };
        let Some(post) = detail.posts.get(self.detail_scroll) else { return };
        let urls: Vec<String> = post.contents.iter().filter_map(|f| match f {
            crate::model::post::ContentFragment::Image { url, .. } => {
                let u = if url.contains("://") { url.clone() } else { format!("{}{}", *crate::constants::BASE_URL, url) };
                Some(u)
            }
            _ => None,
        }).collect();
        if urls.is_empty() {
            let _ = self.action_tx.send(AppAction::ImageData(vec![]));
            return;
        }
        let http = self.http.clone(); let tx = self.action_tx.clone();
        let total = urls.len();
        tokio::spawn(async move {
            let mut data = Vec::new();
            let mut loaded = 0usize;
            for url in &urls {
                match http.get_raw(url).await {
                    Ok(bytes) => {
                        if bytes.len() > 100 {
                            match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    let s = crate::sixel::encode(&img, 320, 240);
                                    if !s.is_empty() { data.push(s); loaded += 1; }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            let _ = tx.send(AppAction::ImageData(data));
        });
    }

    fn check_pending_loads(&mut self) {}

    fn handle_login_key(&mut self, code: KeyCode) {
        if self.login_loading { return; }
        match code {
            KeyCode::Enter => {
                if self.login_username.trim().is_empty() || self.login_password.is_empty() { self.login_error = "请输入用户名和密码".into(); return; }
                self.login_error.clear(); self.login_loading = true;
                self.config.username = self.login_username.clone();
                self.config.password = self.login_password.clone();
                let _ = self.config.save();
                let http = self.http.clone(); let tx = self.action_tx.clone();
                let u = self.login_username.clone(); let p = self.login_password.clone();
                tokio::spawn(async move { let _ = tx.send(AppAction::LoginResult(do_login(&http, &u, &p).await)); });
            }
            KeyCode::Esc => {
                if !self.login_username.is_empty() || !self.login_password.is_empty() { self.login_username.clear(); self.login_password.clear(); self.login_error.clear(); }
                else { self.screen = Screen::Quit; }
            }
            KeyCode::Tab => { self.login_focus = match self.login_focus { LoginField::Username => LoginField::Password, LoginField::Password => LoginField::Username }; }
            KeyCode::Backspace => match self.login_focus { LoginField::Username => { self.login_username.pop(); } LoginField::Password => { self.login_password.pop(); } },
            KeyCode::Char(c) => match self.login_focus { LoginField::Username => { self.login_username.push(c); } LoginField::Password => { self.login_password.push(c); } },
            _ => {}
        }
    }

    fn handle_main_key(&mut self, code: KeyCode) {
        if self.composing { self.handle_compose_key(code); return; }
        if self.search_mode { self.handle_search_key(code); return; }
        match code {
            KeyCode::Char('q') => self.screen = Screen::Quit,
            KeyCode::Tab => {
                if self.show_images && self.detail.is_some() && !self.image_data.is_empty() {
                    self.image_index = (self.image_index + 1) % self.image_data.len();
                    self.set_status(&format!("图片 {}/{} | Tab 下一张  p 关闭 | k/j 移动  f 折叠  r 回复", self.image_index + 1, self.image_data.len()), false);
                } else {
                    self.active_tab = (self.active_tab + 1) % self.tabs.len(); self.detail = None; self.simple_list = SimpleList::default();
                }
            }
            KeyCode::BackTab => { if self.active_tab == 0 { self.active_tab = self.tabs.len() - 1; } else { self.active_tab -= 1; } self.detail = None; }
            KeyCode::Esc => {
                if self.detail.is_some() {
                    self.detail = None; self.detail_scroll = 0; self.show_images = false; self.image_data.clear();
                    let hint = if self.show_list_detail { "d=简洁" } else { "d=详情" };
                    self.set_status(&format!("{} 主题 | k/j=上下 Enter=查看 r=刷新 {} b=板块 q=退出", self.threads.threads.len(), hint), false);
                } else { self.search_mode = false; }
            }
            KeyCode::Up | KeyCode::Char('j') => self.move_up(),
            KeyCode::Down | KeyCode::Char('k') => self.move_down(),
            KeyCode::Right | KeyCode::Char('l') => self.next_page(),
            KeyCode::Left | KeyCode::Char('h') => self.prev_page(),
            KeyCode::PageUp => self.next_page(),
            KeyCode::PageDown => self.prev_page(),
            KeyCode::Enter => self.select_item(),
            KeyCode::Char('p') => {
                if self.detail.is_some() {
                    if self.show_images { self.show_images = false; self.image_data.clear(); self.set_status("k/j 移动  f 折叠  r 回复  p 图片  Esc 返回", false); }
                    else { self.show_images = true; self.image_index = 0; self.image_data.clear(); self.set_status("加载图片中...", false); self.spawn_load_images(); }
                }
            }
            KeyCode::Char('f') => {
                if self.detail.is_some() {
                    self.detail_folded = !self.detail_folded;
                    let state = if self.detail_folded { "折叠" } else { "展开" };
                    self.set_status(&format!("{} | k/j 移动  f 折叠  r 回复  p 图片", if self.detail_folded { "折叠中" } else { "展开中" }), false);
                }
            }
            KeyCode::Char('r') => {
                if self.detail.is_some() || self.simple_list_type == SimpleListType::SmsDetail { self.start_reply(); }
                else if self.detail.is_none() && self.simple_list.items.is_empty() { self.loading = true; self.set_status("刷新中...", false); self.spawn_load_threads(); }
            }
            KeyCode::Char('n') => { if self.detail.is_none() && self.active_tab == 0 { self.start_new_thread(); } }
            KeyCode::Char('d') => {
                if self.detail.is_none() && self.simple_list.items.is_empty() {
                    self.show_list_detail = !self.show_list_detail;
                    let hint = if self.show_list_detail { "d=简洁" } else { "d=详情" };
                    self.set_status(&format!("{} 主题 | k/j=上下 Enter=查看 r=刷新 {} b=板块 q=退出", self.threads.threads.len(), hint), false);
                }
            }
            KeyCode::Char('b') => {
                self.show_forum_panel = !self.show_forum_panel;
                if self.show_forum_panel { self.set_status("版块面板已展开 | b=关闭", false); }
                else {
                    let hint = if self.show_list_detail { "d=简洁" } else { "d=详情" };
                    self.set_status(&format!("{} 主题 | k/j=上下 Enter=查看 r=刷新 {} b=板块 q=退出", self.threads.threads.len(), hint), false);
                }
            }
            KeyCode::Char('/') => { self.search_mode = true; self.search_query.clear(); }
            _ => {}
        }
    }

    fn move_up(&mut self) {
        if self.detail.is_some() { if self.detail_scroll > 0 { self.detail_scroll -= 1; } }
        else if self.simple_list.items.is_empty() { if self.selected_thread > 0 { self.selected_thread -= 1; } }
        else if self.simple_selected > 0 { self.simple_selected -= 1; }
    }
    fn move_down(&mut self) {
        if let Some(ref d) = self.detail { if self.detail_scroll < d.posts.len().saturating_sub(1) { self.detail_scroll += 1; } }
        else if self.simple_list.items.is_empty() { if self.selected_thread < self.threads.threads.len().saturating_sub(1) { self.selected_thread += 1; } }
        else if self.simple_selected < self.simple_list.items.len().saturating_sub(1) { self.simple_selected += 1; }
    }
    fn next_page(&mut self) {
        if let Some(ref d) = self.detail { if self.detail_page < d.last_page { self.detail_page += 1; self.loading = true; self.spawn_load_thread_detail(&self.reply_tid, self.detail_page); } }
        else if self.simple_list.items.is_empty() { if self.thread_page < self.thread_total_pages { self.thread_page += 1; self.loading = true; self.spawn_load_threads(); } }
    }
    fn prev_page(&mut self) {
        if self.detail.is_some() { if self.detail_page > 1 { self.detail_page -= 1; self.loading = true; self.spawn_load_thread_detail(&self.reply_tid, self.detail_page); } }
        else if self.thread_page > 1 { self.thread_page -= 1; self.loading = true; self.spawn_load_threads(); }
    }
    fn select_item(&mut self) {
        if self.simple_list.items.is_empty() {
            if let Some(t) = self.threads.threads.get(self.selected_thread) {
                let tid = t.tid.clone(); self.reply_tid = tid.clone(); self.reply_fid = self.forums[self.selected_forum].0;
                self.detail_page = 1; self.loading = true; self.set_status("加载帖子中...", false); self.spawn_load_thread_detail(&tid, 1);
            }
        } else if let Some(item) = self.simple_list.items.get(self.simple_selected) {
            let tid = item.tid.clone(); self.reply_tid = tid.clone();
            if !item.pid.is_empty() { self.reply_pid = item.pid.clone(); }
            self.detail_page = 1; self.loading = true; self.set_status("加载帖子中...", false); self.spawn_load_thread_detail(&tid, 1);
        }
    }
    fn start_reply(&mut self) { self.composing = true; self.compose_mode = PostMode::ReplyThread; self.compose_content.clear(); self.compose_cursor = 0; }
    fn start_new_thread(&mut self) { self.composing = true; self.compose_mode = PostMode::NewThread; self.compose_content.clear(); self.compose_cursor = 0; }
    fn handle_compose_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => { self.composing = false; self.compose_content.clear(); }
            KeyCode::Enter => { self.composing = false; self.set_status("发送成功!", false); }
            KeyCode::Char(c) => { self.compose_content.push(c); self.compose_cursor += 1; }
            KeyCode::Backspace => { if self.compose_cursor > 0 { self.compose_content.pop(); self.compose_cursor -= 1; } }
            KeyCode::Left => { if self.compose_cursor > 0 { self.compose_cursor -= 1; } }
            KeyCode::Right => { if self.compose_cursor < self.compose_content.len() { self.compose_cursor += 1; } }
            _ => {}
        }
    }
    fn handle_search_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.search_mode = false,
            KeyCode::Enter => { self.search_mode = false; self.set_status(&format!("搜索: {}", self.search_query), false); }
            KeyCode::Char(c) => self.search_query.push(c),
            KeyCode::Backspace => { self.search_query.pop(); }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame) {
        match self.screen { Screen::Login => self.render_login(f), Screen::Main => self.render_main(f), Screen::Quit => {} }
    }

    fn render_login(&self, f: &mut Frame) {
        let area = f.area();
        let pw = area.width.min(60); let ph = 14;
        let pa = Rect::new(area.x + (area.width.saturating_sub(pw)) / 2, area.y + (area.height.saturating_sub(ph)) / 2, pw, ph);
        f.render_widget(Block::default().borders(Borders::ALL).border_style(Theme::block()).title(" HiPDA 登录 ").title_style(Theme::accent()), pa);
        let inner = pa.inner(ratatui::layout::Margin { vertical: 1, horizontal: 2 });
        let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1), Constraint::Length(3), Constraint::Length(2)]).split(inner);
        let cursor = if self.login_loading { "" } else { "▌" };
        let field_style = |focused: bool| {
            if focused && !self.login_loading { Theme::accent() } else { Theme::text_dim() }
        };
        let border_style = |focused: bool| {
            if focused && !self.login_loading { Theme::accent() } else { Theme::block() }
        };
        f.render_widget(
            Paragraph::new(format!("{} {}", self.login_username, if self.login_focus == LoginField::Username { cursor } else { "" }))
                .block(Block::default().borders(Borders::ALL).border_style(border_style(self.login_focus == LoginField::Username)).title(" 用户名 ").title_style(field_style(self.login_focus == LoginField::Username)))
                .style(field_style(self.login_focus == LoginField::Username)),
            rows[1]);
        let mask: String = self.login_password.chars().map(|_| '•').collect();
        f.render_widget(
            Paragraph::new(format!("{} {}", mask, if self.login_focus == LoginField::Password { cursor } else { "" }))
                .block(Block::default().borders(Borders::ALL).border_style(border_style(self.login_focus == LoginField::Password)).title(" 密码 ").title_style(field_style(self.login_focus == LoginField::Password)))
                .style(field_style(self.login_focus == LoginField::Password)),
            rows[3]);
        let hint = if self.login_loading { Line::from(Span::styled("登录中...", Theme::yellow())) }
        else if !self.login_error.is_empty() { Line::from(Span::styled(self.login_error.as_str(), Theme::red())) }
        else { Line::from(vec![Span::styled(" Enter ", Theme::selected()), Span::styled(" 登录  ", Theme::text_dim()), Span::styled(" Tab ", Theme::selected()), Span::styled(" 切换  ", Theme::text_dim()), Span::styled(" Esc ", Theme::selected()), Span::styled(" 退出", Theme::text_dim())]) };
        f.render_widget(Paragraph::new(hint).centered(), rows[4]);
    }

    fn render_main(&self, f: &mut Frame) {
        let area = f.area();
        let layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]).split(area);
        let tab_items: Vec<Line> = self.tabs.iter().enumerate().map(|(i, t)| {
            if i == self.active_tab { Line::from(Span::styled(format!(" {} ", t), Theme::tab_active())) }
            else { Line::from(Span::styled(format!(" {} ", t), Theme::tab_inactive())) }
        }).collect();
        f.render_widget(Tabs::new(tab_items).style(Theme::text_dim()), layout[0]);
        let main_layout = Layout::default().direction(Direction::Horizontal).constraints(
            if self.detail.is_some() || !self.simple_list.items.is_empty() { vec![Constraint::Min(0)] }
            else if self.show_forum_panel { vec![Constraint::Length(14), Constraint::Min(0)] }
            else { vec![Constraint::Min(0)] }
        ).split(layout[1]);
        if self.detail.is_none() && self.simple_list.items.is_empty() {
            if self.show_forum_panel { self.render_forum_list(f, main_layout[0]); self.render_thread_list(f, main_layout[1]); }
            else { self.render_thread_list(f, main_layout[0]); }
        } else if let Some(ref d) = self.detail { self.render_thread_detail(f, main_layout[0], d); }
        else { self.render_simple_list(f, main_layout[0]); }
        if self.composing { self.render_compose(f, area); }
        if self.search_mode { self.render_search(f, area); }
        let ss = if self.status_error { Theme::status_error() } else { Theme::status_normal() };
        let status_text = if self.loading { " 加载中... ".to_string() } else { format!(" {} ", self.status_msg) };
        f.render_widget(Paragraph::new(Line::from(Span::styled(status_text, ss))), layout[2]);
    }

    fn render_forum_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.forums.iter().enumerate().map(|(i, (_, name))| {
            let label = if i == self.selected_forum { format!("  {} ", name) } else { format!("  {} ", name) };
            if i == self.selected_forum { ListItem::new(Line::from(Span::styled(label, Theme::selected()))) }
            else { ListItem::new(Line::from(Span::styled(label, Theme::text()))) }
        }).collect();
        f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Theme::block()).title(" 版块 ").title_style(Theme::text_dim())), area);
    }

    fn render_thread_list(&self, f: &mut Frame, area: Rect) {
        let inner_w = area.width.saturating_sub(4) as usize;
        let visible = area.height.saturating_sub(2) as usize;
        let total = self.threads.threads.len();
        let mut scroll = self.list_scroll.get();
        if self.selected_thread < scroll { scroll = self.selected_thread; }
        else if self.selected_thread >= scroll.saturating_add(visible) { scroll = self.selected_thread.saturating_sub(visible.saturating_sub(1)); }
        if scroll + visible > total { scroll = total.saturating_sub(visible); }
        self.list_scroll.set(scroll);
        let items: Vec<ListItem> = self.threads.threads.iter().enumerate().skip(scroll).take(visible).map(|(i, t)| {
            let prefix = if i == self.selected_thread { ">" } else { " " };
            let count_str: String = t.count_cmts.chars().take(4).collect();
            let icon = if t.is_poll { "\u{f080}" } else if t.with_pic { "\u{f03e}" } else { " " };
            let left = format!("{}[{:>4}] ", prefix, count_str);
            let line = if self.show_list_detail {
                let author: String = t.author.chars().take(6).collect();
                let tl = t.time_create.chars().count();
                let time = if tl > 5 { t.time_create.chars().skip(tl - 5).collect::<String>() } else { t.time_create.clone() };
                let right = format!("{}  \u{f007} {:>6}  \u{f073} {}", icon, author, time);
                let tmax = 35usize; let tt: String = t.title.chars().take(tmax).collect();
                let mid = format!("{}{}  {}", left, tt, right);
                let pad = inner_w.saturating_sub(mid.chars().count());
                format!("{}{}", mid, " ".repeat(pad))
            } else {
                let tt: String = t.title.chars().take(50).collect();
                let mid = format!("{}{} {}", left, tt, icon);
                let pad = inner_w.saturating_sub(mid.chars().count());
                format!("{}{}", mid, " ".repeat(pad))
            };
            if i == self.selected_thread { ListItem::new(Line::from(Span::styled(line, Theme::selected_dim()))) }
            else { ListItem::new(Line::from(Span::styled(line, Theme::text()))) }
        }).collect();
        let fnm = self.forums.get(self.selected_forum).map(|f| f.1.as_str()).unwrap_or("?");
        f.render_widget(List::new(items).block(
            Block::default().borders(Borders::ALL).border_style(Theme::block())
                .title(format!(" {} (页 {}) ", fnm, self.thread_page)).title_style(Theme::text_dim())
        ), area);
    }

    fn render_thread_detail(&self, f: &mut Frame, area: Rect, detail: &ThreadDetail) {
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::styled(format!(" {}", detail.title), Theme::accent_bold())));
        let fh = if self.detail_folded { "f=展开" } else { "f=折叠" };
        lines.push(Line::from(Span::styled(format!(" 页 {}/{} | k/j 移动  h/l 翻页  {}  r 回复  p 图片  Esc 返回", detail.page, detail.last_page, fh), Theme::text_dim())));
        lines.push(Line::from(""));
        for (i, post) in detail.posts.iter().enumerate() {
            let ind = if i == self.detail_scroll { "▸" } else { " " };
            lines.push(Line::from(Span::styled(format!(" {} #{:02}  {}  {}", ind, post.floor, post.author, post.time_post),
                if i == self.detail_scroll { Theme::accent() } else { Theme::text_muted() })));
            if !self.detail_folded || i == self.detail_scroll {
                lines.push(Line::from(""));
                for frag in &post.contents {
                    match frag {
                        crate::model::post::ContentFragment::Text { text, bold, color, .. } => {
                            let mut s = Theme::text(); if *bold { s = s.add_modifier(Modifier::BOLD); } if !color.is_empty() { s = s.fg(str_to_color(color)); }
                            lines.push(Line::from(Span::styled(format!("  {}", text), s)));
                        }
                        crate::model::post::ContentFragment::Link { text, .. } => { lines.push(Line::from(Span::styled(format!("  ↳ {}", text), Theme::blue()))); }
                        crate::model::post::ContentFragment::Image { url, .. } => { lines.push(Line::from(Span::styled(format!("  ◉ {}", url), Theme::green()))); }
                        crate::model::post::ContentFragment::Quote { author_and_time, .. } => { lines.push(Line::from(Span::styled(format!("  ┃ {}", author_and_time), Theme::text_dim()))); }
                        crate::model::post::ContentFragment::LineBreak => { lines.push(Line::from("")); }
                        crate::model::post::ContentFragment::Notice(msg) => { lines.push(Line::from(Span::styled(format!("  {}", msg), Theme::text_dim()))); }
                        _ => {}
                    }
                }
                lines.push(Line::from(""));
            }
        }
        let visible = area.height.saturating_sub(3) as usize;
        let total_lines = lines.len();
        let mut sel_line = 0usize; let mut lc = 0usize;
        for (pi, post) in detail.posts.iter().enumerate() {
            if pi == self.detail_scroll { sel_line = lc; }
            lc += 1;
            if !self.detail_folded || pi == self.detail_scroll { lc += 1 + post.contents.len() + 1; }
        }
        let ph = detail.posts.get(self.detail_scroll).map(|p| 3 + p.contents.len()).unwrap_or(1);
        let scroll = if sel_line + ph <= visible { 0 } else { let mid = sel_line.saturating_sub(visible / 2); mid.min(total_lines.saturating_sub(visible)) };
        let end = (scroll + visible).min(total_lines);
        let vl: Vec<Line> = lines[scroll..end].to_vec();
        let block = Block::default().borders(Borders::ALL).border_style(Theme::block())
            .title(format!(" 帖子 ({} 回复) ", detail.posts.len())).title_style(Theme::text_dim());
        f.render_widget(block, area);
        f.render_widget(Paragraph::new(vl), area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 }));
        if self.show_images {
            // Position image at right edge inside the block, aligned to col 0 for sixel compat
            let col = area.x + area.width.saturating_sub(42).max(area.x + 2);
            let row = area.y + area.height.saturating_sub(12).max(area.y + 2);
            self.img_area.set(Some((col, row)));
        }
    }

    fn render_simple_list(&self, f: &mut Frame, area: Rect) {
        let title = match self.simple_list_type {
            SimpleListType::Search => "搜索结果", SimpleListType::MyPosts => "我的帖子", SimpleListType::MyReplies => "我的回复",
            SimpleListType::Favorites => "收藏", SimpleListType::Attention => "关注", SimpleListType::Sms => "短消息",
            SimpleListType::SmsDetail => "短消息详情", SimpleListType::Notify => "通知",
        };
        let items: Vec<ListItem> = self.simple_list.items.iter().enumerate().map(|(i, item)| {
            let p = if i == self.simple_selected { "▸" } else { " " };
            let t = format!(" {} {} | {} | {}", p, item.title, item.author, item.time);
            if i == self.simple_selected { ListItem::new(Line::from(Span::styled(t, Theme::accent()))) }
            else { ListItem::new(Line::from(Span::styled(t, Theme::text()))) }
        }).collect();
        f.render_widget(List::new(items).block(
            Block::default().borders(Borders::ALL).border_style(Theme::block()).title(format!(" {} ", title)).title_style(Theme::text_dim())
        ), area);
    }

    fn render_compose(&self, f: &mut Frame, area: Rect) {
        let pa = centered_rect(area, 70, 60);
        let c = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Min(3)]).split(pa);
        let t = match self.compose_mode { PostMode::ReplyThread => "回复", PostMode::ReplyPost => "回复楼层", PostMode::QuotePost => "引用回复", PostMode::NewThread => "发表新帖", PostMode::EditPost => "编辑", };
        f.render_widget(Paragraph::new(self.compose_content.as_str()).style(Theme::text())
            .block(Block::default().borders(Borders::ALL).border_style(Theme::accent()).title(format!(" {} ", t)).title_style(Theme::accent())), c[1]);
    }

    fn render_search(&self, f: &mut Frame, area: Rect) {
        let pa = centered_rect(area, 50, 10);
        let c = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Length(3)]).split(pa);
        f.render_widget(Paragraph::new(format!(" 🔍 {}▌", self.search_query)).style(Theme::text())
            .block(Block::default().borders(Borders::ALL).border_style(Theme::accent()).title(" 搜索 ").title_style(Theme::accent())), c[1]);
    }
}

fn str_to_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "red" => Color::Red, "green" => Color::Green, "blue" => Color::Blue, "yellow" => Color::Yellow,
        "cyan" => Color::Cyan, "magenta" => Color::Magenta, "white" => Color::White, "gray" | "grey" => Color::Gray,
        _ => Color::White,
    }
}

fn centered_rect(r: Rect, px: u16, py: u16) -> Rect {
    let w = r.width * px / 100; let h = r.height * py / 100;
    Rect::new(r.x + (r.width - w) / 2, r.y + (r.height - h) / 2, w, h)
}
