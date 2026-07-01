pub fn get_middle_string(source: &str, start: &str, end: &str) -> String {
    let s = source.find(start).map(|i| i + start.len()).unwrap_or(0);
    let e = if end.is_empty() { source.len() } else { source[s..].find(end).map(|i| s + i).unwrap_or(source.len()) };
    if e <= s { return String::new(); }
    source[s..e].to_string()
}

pub fn get_int_from_string(s: &str) -> i32 { s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0) }
pub fn parse_int(s: &str) -> i32 { s.trim().parse().unwrap_or(0) }
pub fn parse_size_text(size_text: &str) -> i64 {
    let t = size_text.trim().to_uppercase();
    if let Some(n) = t.strip_suffix("KB") { return (n.trim().parse::<f64>().unwrap_or(0.0) * 1024.0) as i64; }
    if let Some(n) = t.strip_suffix("MB") { return (n.trim().parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0) as i64; }
    if let Some(n) = t.strip_suffix("BYTES") { return n.trim().parse().unwrap_or(0); }
    -1
}
