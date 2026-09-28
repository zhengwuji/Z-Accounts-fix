//! Claimable-plan (领取) flow: captcha config, preview, claim with Aliyun
//! captcha headers, activation reporting, and business-code mapping.

use crate::httpc::{agent, zcode_headers, ZCODE_BASE};
use crate::i18n::{t, Lang};
use serde_json::{json, Value};

pub struct ClaimPlan {
    pub plan_id: String,
    pub name: String,
    pub description: String,
    pub grants: Vec<String>,
    pub grant_items: Vec<(String, f64, String)>, // name, units, period
}

impl ClaimPlan {
    pub fn to_json(&self) -> Value {
        json!({
            "plan_id": self.plan_id,
            "name": self.name,
            "description": self.description,
            "grants": self.grants,
            "grant_items": self.grant_items.iter()
                .map(|(n, u, p)| json!({"name": n, "units": u, "period": p}))
                .collect::<Vec<_>>(),
        })
    }
}

fn app_params() -> String {
    let av = crate::httpc::zcode_client_version();
    format!(
        "app_version={}&platform=win32",
        crate::httpc::urlencode(&av)
    )
}

fn call_json(
    call: Result<ureq::Response, ureq::Error>,
    lang: Lang,
    fallback: &str,
) -> Result<Value, String> {
    match call {
        Ok(r) => r
            .into_json::<Value>()
            .map_err(|e| { let es = e.to_string(); t(lang, "err.http.read", &[("e", &es)]) }),
        Err(ureq::Error::Status(c, r)) => {
            let body = r.into_string().unwrap_or_default();
            if let Ok(doc) = serde_json::from_str::<Value>(&body) {
                return Ok(doc); // business-level errors travel inside 200-ish bodies
            }
            Err(t(lang, fallback, &[("code", &c.to_string()), ("msg", &body.chars().take(120).collect::<String>())]))
        }
        Err(e) => { let es = e.to_string(); Err(t(lang, "err.network", &[("e", &es)])) }
    }
}

/// GET /api/v1/client/configs → data.configs.captcha
pub fn captcha_config(lang: Lang) -> Result<Value, String> {
    let url = format!("{ZCODE_BASE}/api/v1/client/configs");
    let doc = call_json(
        crate::httpc::zcode_source_headers(agent().get(&url)).call(),
        lang,
        "err.claim.config_req",
    )?;
    let cap = doc
        .pointer("/data/configs/captcha")
        .cloned()
        .unwrap_or(Value::Null);
    let enabled = cap.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let scene_id = cap
        .get("sceneId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !enabled || scene_id.is_empty() {
        return Err(t(lang, "err.claim.config_unavailable", &[]));
    }
    Ok(json!({
        "enabled": enabled,
        "scene_id": scene_id,
        "region": cap.get("region").and_then(|v| v.as_str()).unwrap_or(""),
        "prefix": cap.get("prefix").and_then(|v| v.as_str()).unwrap_or(""),
    }))
}

/// Activation report — the claim flow expects the client to have reported
/// app_launch / app_daily_active events first (POST /api/v1/event/report).
pub fn report_activation(token: &str, lang: Lang) -> Result<(), String> {
    let url = format!("{ZCODE_BASE}/api/v1/event/report");
    let now_ms = chrono::Utc::now().timestamp_millis();
    let body = json!({
        "events": [
            { "event": "app_launch", "timestamp": now_ms, "context": {
                "platform": "win32", "app_version": crate::httpc::app_version_string(),
                "device_os_category": "windows", "device_os_version": std::env::consts::OS,
                "screen_resolution": "", "marketing_params": "", "event_extra_details": "" } },
            { "event": "app_daily_active", "timestamp": now_ms, "context": {
                "platform": "win32", "app_version": crate::httpc::app_version_string() } }
        ]
    });
    let resp = zcode_headers(
        agent()
            .post(&url)
            .set("Authorization", &format!("Bearer {token}")),
        &crate::httpc::app_version_string(),
    )
    .send_json(body);
    match resp {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(c, r)) => {
            let _ = r.into_string();
            Err(t(lang, "err.claim.activate_req", &[("e", &format!("HTTP {c}"))]))
        }
        Err(e) => { let es = e.to_string(); Err(t(lang, "err.claim.activate_req", &[("e", &es)])) }
    }
}

/// GET /api/v1/zcode-plan/billing/preview → data.plans[]
pub fn preview(token: &str, lang: Lang) -> Result<Vec<ClaimPlan>, String> {
    let url = format!(
        "{ZCODE_BASE}/api/v1/zcode-plan/billing/preview?{}",
        app_params()
    );
    let doc = call_json(
        crate::httpc::zcode_api_get(&url, token).call(),
        lang,
        "err.claim.preview_req",
    )?;
    check_business(&doc, lang)?;
    let arr = doc
        .pointer("/data/plans")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for p in &arr {
        let plan_id = p
            .get("plan_id")
            .or_else(|| p.get("planId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if plan_id.is_empty() {
            continue;
        }
        let mut grant_items = Vec::new();
        if let Some(items) = p.get("grant_items").or_else(|| p.get("grantItems")).and_then(|v| v.as_array()) {
            for gi in items {
                grant_items.push((
                    gi.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                    gi.get("units").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    gi.get("period").and_then(|v| v.as_str()).unwrap_or("one_time").into(),
                ));
            }
        }
        let grants: Vec<String> = p
            .get("grants")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|g| g.as_str().map(String::from)).collect())
            .unwrap_or_default();
        out.push(ClaimPlan {
            plan_id,
            name: p.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
            description: p
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .into(),
            grants,
            grant_items,
        });
    }
    Ok(out)
}

fn check_business(doc: &Value, lang: Lang) -> Result<(), String> {
    let code = doc.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    if code == 0 || code == 200 {
        return Ok(());
    }
    let msg = doc
        .get("msg")
        .or_else(|| doc.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let base = claim_fail_message(lang, code);
    if msg.is_empty() {
        Err(base)
    } else {
        Err(t(lang, "claim.fail.with_server", &[("base", &base), ("server_msg", msg)]))
    }
}

pub fn claim_fail_message(lang: Lang, code: i64) -> String {
    let key = match code {
        1001 => "claim.fail.1001",
        1002 => "claim.fail.1002",
        1003 => "claim.fail.1003",
        1004 => "claim.fail.1004",
        1005 => "claim.fail.1005",
        3001 => "claim.fail.3001",
        3007 => "claim.fail.3007",
        401 => "claim.fail.401",
        _ => "claim.fail.generic",
    };
    t(lang, key, &[])
}

pub struct ClaimOutcome {
    pub ok: bool,
    pub code: Option<i64>,
    pub message: String,
    pub plan_name: String,
    pub starts_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub server_time: Option<i64>,
    pub next_at: Option<i64>,
    pub server_msg: String,
}

/// POST /api/v1/zcode-plan/billing/claim with Aliyun captcha verify headers.
pub fn claim(
    token: &str,
    plan_id: &str,
    captcha_param: Option<&str>,
    captcha_region: Option<&str>,
    lang: Lang,
) -> Result<ClaimOutcome, String> {
    let url = format!("{ZCODE_BASE}/api/v1/zcode-plan/billing/claim");
    let mut req = crate::httpc::zcode_api_post(&url, token);
    let ver = crate::httpc::zcode_client_version();
    req = req
        .set("X-ZCode-App-Version", &ver)
        .set("X-Platform", "win32");
    if let (Some(p), Some(r)) = (captcha_param, captcha_region) {
        if !p.is_empty() {
            req = req
                .set("X-Aliyun-Captcha-Verify-Param", p)
                .set("X-Aliyun-Captcha-Verify-Region", r);
        }
    }
    let doc = call_json(
        req.send_json(json!({ "plan_id": plan_id })),
        lang,
        "err.claim.claim_req",
    )?;
    let code = doc.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    let server_msg = doc
        .pointer("/data/message")
        .or_else(|| doc.get("msg"))
        .or_else(|| doc.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let server_time = doc
        .pointer("/data/server_time")
        .and_then(|v| v.as_i64());
    let starts_at = doc.pointer("/data/starts_at").and_then(|v| v.as_i64());
    let ends_at = doc.pointer("/data/ends_at").and_then(|v| v.as_i64());
    let ok = code == 0 || code == 200;
    let message = if ok {
        String::new()
    } else {
        let base = claim_fail_message(lang, code);
        if server_msg.is_empty() {
            base
        } else {
            t(lang, "claim.fail.with_server", &[("base", &base), ("server_msg", &server_msg)])
        }
    };
    // 1005 = daily cap reached; response carries the next eligible time
    let next_at = if code == 1005 {
        doc.pointer("/data/next_at")
            .or_else(|| doc.pointer("/data/nextAt"))
            .and_then(|v| v.as_i64())
    } else {
        None
    };
    Ok(ClaimOutcome {
        ok,
        code: Some(code),
        message,
        plan_name: String::new(),
        starts_at,
        ends_at,
        server_time,
        next_at,
        server_msg,
    })
}
