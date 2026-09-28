//! Shared HTTP helpers (ureq, rustls-backed, spoofed client UA/headers).

use std::time::Duration;

pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0";

pub const ZCODE_BASE: &str = "https://zcode.z.ai";
pub const BIGMODEL_BASE: &str = "https://open.bigmodel.cn";
pub const BIGMODEL_WWW: &str = "https://bigmodel.cn";
pub const ZAI_API_BASE: &str = "https://api.z.ai";
pub const ZAI_CHAT_BASE: &str = "https://chat.z.ai";

pub fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .user_agent(UA)
        .build()
}

pub fn zcode_headers(req: ureq::Request, app_version: &str) -> ureq::Request {
    req.set("X-ZCode-App-Version", app_version)
        .set("X-Release-Channel", "stable")
        .set("X-Client-Language", "zh-CN")
        .set("X-Client-Timezone", "Asia/Shanghai")
}

pub fn app_version_string() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// App version string reported to the zcode.z.ai API. The endpoints validate
/// the header/query pair against a client-style version (ZCode desktop 3.x),
/// so prefer the locally installed ZCode version over our own.
pub fn zcode_client_version() -> String {
    if let Ok(v) = std::env::var("ZCODE_APP_VERSION") {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    "3.14.3".to_string()
}

/// The header set ZCode clients send to zcode.z.ai ("source headers") —
/// required by the billing endpoints alongside the query `app_version`.
pub fn zcode_source_headers(req: ureq::Request) -> ureq::Request {
    let ver = zcode_client_version();
    let mid = crate::zcode::current_device_mid().unwrap_or_default();
    let req = req
        .set("User-Agent", &format!("ZCode/{ver}"))
        .set("HTTP-Referer", ZCODE_BASE)
        .set("X-Title", "Z Code@electron")
        .set("X-ZCode-App-Version", &ver)
        .set("X-Platform", "win32-x64")
        .set("X-Release-Channel", "stable")
        .set("X-Client-Language", "zh-CN")
        .set("X-Client-Timezone", "Asia/Shanghai")
        .set("X-Os-Category", "windows")
        .set("X-Os-Version", "10.0.26200");
    if mid.is_empty() {
        req
    } else {
        req.set("X-Device-Mid", &mid)
    }
}

/// Authorized request to zcode.z.ai with the full source-header set.
pub fn zcode_api_get(url: &str, bearer: &str) -> ureq::Request {
    zcode_source_headers(agent().get(url)).set("Authorization", &format!("Bearer {bearer}"))
}

pub fn zcode_api_post(url: &str, bearer: &str) -> ureq::Request {
    zcode_source_headers(agent().post(url))
        .set("Authorization", &format!("Bearer {bearer}"))
        .set("Content-Type", "application/json")
}

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
