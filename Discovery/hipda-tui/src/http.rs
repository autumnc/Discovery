use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE, SET_COOKIE, USER_AGENT};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    cookies: Arc<Mutex<HashMap<String, String>>>,
}

impl HttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str("net.jejer.hipda 1.0.0")?);
        Ok(Self {
            client: Client::builder().default_headers(headers).timeout(std::time::Duration::from_secs(10)).build()?,
            cookies: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_cookie_string(&self) -> String {
        self.cookies.lock().unwrap().iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("; ")
    }

    pub fn clear_cookies(&self) { self.cookies.lock().unwrap().clear(); }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let cookie_str = self.get_cookie_string();
        let resp = self.client.get(url).header(COOKIE, &cookie_str).send().await?;
        self.store_cookies(&resp);
        decode_response(resp).await
    }

    pub async fn get_raw(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let cookie_str = self.get_cookie_string();
        let resp = self.client.get(url).header(COOKIE, &cookie_str).send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn post(&self, url: &str, params: &HashMap<String, String>) -> anyhow::Result<String> {
        let cookie_str = self.get_cookie_string();
        let resp = self.client.post(url).header(COOKIE, &cookie_str).form(params).send().await?;
        self.store_cookies(&resp);
        decode_response(resp).await
    }

    fn store_cookies(&self, resp: &reqwest::Response) {
        let mut cookies = self.cookies.lock().unwrap();
        for header in resp.headers().get_all(SET_COOKIE) {
            if let Ok(s) = header.to_str() {
                if let Some(part) = s.split(';').next() {
                    if let Some(eq) = part.find('=') {
                        cookies.insert(part[..eq].trim().into(), part[eq + 1..].trim().into());
                    }
                }
            }
        }
    }
}

async fn decode_response(resp: reqwest::Response) -> anyhow::Result<String> {
    let encoding = resp.headers().get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| if ct.to_uppercase().contains("UTF") { "UTF-8" } else { "GBK" })
        .unwrap_or("GBK");
    let bytes = resp.bytes().await?;
    if encoding == "UTF-8" { Ok(String::from_utf8(bytes.to_vec())?) }
    else { Ok(encoding_rs::GBK.decode(&bytes).0.into_owned()) }
}
