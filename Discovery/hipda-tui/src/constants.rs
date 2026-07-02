use std::sync::LazyLock;

pub static BASE_URL: LazyLock<String> = LazyLock::new(|| "https://www.4d4y.com/forum/".into());
pub static LOGIN_SUBMIT: LazyLock<String> = LazyLock::new(|| format!("{}logging.php?action=login&loginsubmit=yes", *BASE_URL));
pub static LOGIN_GET_FORMHASH: LazyLock<String> = LazyLock::new(|| format!("{}logging.php?action=login", *BASE_URL));

#[derive(Debug, Clone)]
pub struct Forum { pub id: i32, pub name: &'static str }

pub const FID_DISCOVERY: i32 = 2;
pub const FID_BS: i32 = 6;

pub const FORUMS: &[Forum] = &[
    Forum { id: FID_DISCOVERY, name: "Discovery" },
    Forum { id: FID_BS, name: "Buy & Sell" },
    Forum { id: 7, name: "Geek Talks" },
    Forum { id: 59, name: "E-INK" },
    Forum { id: 12, name: "PalmOS" },
    Forum { id: 57, name: "疑似机器人" },
    Forum { id: 63, name: "已完成交易" },
    Forum { id: 62, name: "Joggler" },
    Forum { id: 5, name: "站务与公告" },
    Forum { id: 9, name: "Smartphone" },
    Forum { id: 56, name: "iPhone" },
    Forum { id: 60, name: "Android" },
    Forum { id: 14, name: "Windows Mobile" },
    Forum { id: 22, name: "麦客爱苹果" },
    Forum { id: 50, name: "DC,NB,MP3" },
    Forum { id: 24, name: "意欲蔓延" },
    Forum { id: 23, name: "随笔与个人文集" },
    Forum { id: 25, name: "吃喝玩乐" },
    Forum { id: 51, name: "La Femme" },
    Forum { id: 65, name: "改版建议" },
    Forum { id: 64, name: "只讨论2.0" },
];

pub const DEFAULT_FORUMS: &[i32] = &[FID_DISCOVERY, FID_BS, 7];

pub fn get_forum_by_fid(fid: i32) -> Option<&'static Forum> { FORUMS.iter().find(|f| f.id == fid) }

// Tab endpoint URLs (matching Discuz! 7.2 / HiPDA Android app)
pub const URL_MY_POSTS: &str = "my.php?item=threads";
pub const URL_MY_REPLIES: &str = "my.php?item=posts";
pub const URL_FAVORITES: &str = "my.php?item=favorites&type=thread";
pub const URL_ATTENTION: &str = "my.php?item=buddylist&type=thread";
pub const URL_SMS: &str = "pm.php?filter=privatepm";
pub const URL_NOTIFY: &str = "notice.php";
pub const URL_SEARCH: &str = "search.php?srchtype={srchtype}&srchtxt={srchtxt}&searchsubmit=true&st=on&srchuname={srchuname}&srchfilter=all&srchfrom=0&before=&orderby=lastpost&ascdesc=desc&srchfid%5B0%5D={fid}";
