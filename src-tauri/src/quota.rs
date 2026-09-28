//! Quota queries — GET zcode.z.ai/api/v1/zcode-plan/billing/balance
//! (?app_version=) authenticated with the account's `zcodejwttoken` and the
//! full ZCode client source-header set. Response: data.plans[] (plan
//! containers + entitlement grants) and data.balances[] (per-model usage
//! meters: total/used/remaining units).

use crate::httpc::{agent, BIGMODEL_WWW, UA};
use crate::i18n::Lang;
use serde_json::{json, Value};

#[derive(Default, Clone)]
pub struct QuotaItem {
    pub name: String,
    pub unit: String,
    pub unit_code: String,
    pub kind: String, // prompt_count | duration | raw | grant
    pub total: Option<f64>,
    pub used: Option<f64>,
    pub remaining: Option<f64>,
    pub percent_used: Option<f64>,
    pub window: Option<String>,
    pub period_end: String,
    pub reset: String,
}

impl QuotaItem {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name, "unit": self.unit, "unit_code": self.unit_code,
            "kind": self.kind,
            "total": self.total, "used": self.used, "remaining": self.remaining,
            "percent_used": self.percent_used, "window": self.window,
            "period_end": self.period_end, "reset": self.reset,
        })
    }
}

#[derive(Default, Clone)]
pub struct QuotaPlan {
    pub tier: Option<String>,
    pub tier_code: Option<String>,
    pub name: String,
    pub expire: Option<String>,
    pub pid: Option<String>,
    pub items: Vec<QuotaItem>,
}

impl QuotaPlan {
    fn to_json(&self) -> Value {
        json!({
            "tier": self.tier, "tier_code": self.tier_code, "name": self.name,
            "expire": self.expire, "pid": self.pid,
            "items": self.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Default)]
pub struct QuotaData {
    pub items: Vec<QuotaItem>,
    pub plans: Vec<QuotaPlan>,
    pub plan_tier: Option<String>,
    pub total: Option<f64>,
    pub is_empty: bool,
    pub plan_expire: Option<String>,
}

impl QuotaData {
    pub fn to_json(&self, refreshed_at: i64) -> Value {
        json!({
            "items": self.items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
            "plans": self.plans.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
            "plan_tier": self.plan_tier,
            "total": self.total,
            "is_empty": self.is_empty,
            "plan_expire": self.plan_expire,
            "refreshed_at": refreshed_at,
        })
    }
}

fn num(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.replace(',', "").parse::<f64>().ok()))
        .filter(|n| n.is_finite())
}

fn biz_code(v: &Value) -> Option<i64> {
    v.get("code").and_then(|c| c.as_i64())
}

fn biz_msg(v: &Value) -> String {
    v.get("msg")
        .or_else(|| v.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string()
}

fn http_error(lang: Lang, code: u16, body: &str) -> String {
    match code {
        401 => crate::i18n::t(lang, "err.token.http401", &[("code", "401")]),
        429 => crate::i18n::t(lang, "err.quota.http429", &[]),
        _ => crate::i18n::t(
            lang,
            "err.quota.http",
            &[("code", &code.to_string()), ("msg", &body.chars().take(120).collect::<String>())],
        ),
    }
}

// ------------------------------------------------------------ bigmodel side

// ---------------------------------------------------------- bigmodel identity

/// GET https://bigmodel.cn/api/biz/customer/getCustomerInfo (verified live).
pub fn bigmodel_customer_info(token: &str) -> Option<Value> {
    let url = format!("{BIGMODEL_WWW}/api/biz/customer/getCustomerInfo");
    let resp = agent()
        .get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", UA)
        .call()
        .ok()?;
    let doc: Value = resp.into_json().ok()?;
    if biz_code(&doc).unwrap_or(0) != 200 {
        return None;
    }
    doc.get("data").cloned()
}

// ---------------------------------------------------------------- z.ai side

fn fmt_secs(secs: i64) -> String {
    use chrono::TimeZone;
    match chrono::Local.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(t) => t.format("%Y-%m-%d %H:%M").to_string(),
        _ => String::new(),
    }
}

/// GET /api/v1/zcode-plan/billing/balance?app_version= — the single quota
/// endpoint used for every account family (Bearer = zcodejwttoken).
fn zai_balance(token: &str, lang: Lang) -> Result<QuotaData, String> {
    let ver = crate::httpc::zcode_client_version();
    let url = format!(
        "{}/api/v1/zcode-plan/billing/balance?app_version={}",
        crate::httpc::ZCODE_BASE,
        crate::httpc::urlencode(&ver)
    );
    let resp = crate::httpc::zcode_api_get(&url, token).call();
    let doc: Value = match resp {
        Ok(r) => r.into_json().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(c, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(http_error(lang, c, &body));
        }
        Err(e) => {
            let es = e.to_string();
            return Err(crate::i18n::t(lang, "err.network", &[("e", &es)]));
        }
    };
    if let Some(code) = biz_code(&doc) {
        if code == 401 {
            return Err(crate::i18n::t(lang, "err.token.biz401", &[]));
        }
        if code != 0 && code != 200 {
            return Err(crate::i18n::t(
                lang,
                "err.quota.biz",
                &[("code", &code.to_string()), ("msg", &biz_msg(&doc))],
            ));
        }
    }
    let data = doc.get("data").cloned().unwrap_or(json!({}));

    // plans[] → plan containers (tier/name/expire + entitlement grants)
    let mut out = QuotaData::default();
    let now_secs = chrono::Utc::now().timestamp();
    let mut expired_user_plans: Vec<String> = Vec::new();

    // balances first — they carry the per-model usage meters
    struct Bal {
        user_plan_id: String,
        plan_id: String,
        item: QuotaItem,
    }
    let mut balances: Vec<Bal> = Vec::new();
    let mut saw_any = false;
    if let Some(arr) = data.get("balances").and_then(|v| v.as_array()) {
        for b in arr {
            let total = b.get("total_units").and_then(|v| num(v));
            let used = b.get("used_units").and_then(|v| num(v));
            let remaining = b
                .get("remaining_units")
                .or_else(|| b.get("available_units"))
                .and_then(|v| num(v));
            let percent = match (total, used) {
                (Some(t), Some(u)) if t > 0.0 => Some(u * 100.0 / t),
                _ => None,
            };
            let period_end = b
                .get("expires_at")
                .or_else(|| b.get("period_end"))
                .and_then(|v| v.as_i64())
                .filter(|&s| s > 0)
                .map(fmt_secs)
                .unwrap_or_default();
            let unit = b.get("unit_type").and_then(|v| v.as_str()).unwrap_or("token");
            let item = QuotaItem {
                name: b.get("show_name").and_then(|v| v.as_str()).unwrap_or("usage").to_string(),
                unit: unit.to_string(),
                unit_code: unit.to_string(),
                kind: "raw".to_string(),
                total,
                used,
                remaining,
                percent_used: percent,
                window: None,
                period_end,
                reset: String::new(),
            };
            balances.push(Bal {
                user_plan_id: b.get("user_plan_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                plan_id: b.get("plan_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                item,
            });
            saw_any = true;
        }
    }
    if let Some(plans) = data.get("plans").and_then(|v| v.as_array()) {
        for p in plans {
            let plan_id = p.get("plan_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let user_plan_id = p
                .get("user_plan_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let ends_at = p.get("ends_at").and_then(|v| v.as_i64()).unwrap_or(0);
            let effectively_expired =
                status == "expired" || (status == "active" && ends_at > 0 && ends_at <= now_secs);
            if effectively_expired && !user_plan_id.is_empty() {
                expired_user_plans.push(user_plan_id.clone());
            }
            let pid_l = plan_id.to_lowercase();
            let tier = if pid_l.contains("max") {
                Some("Max".to_string())
            } else if pid_l.contains("pro") {
                Some("Pro".to_string())
            } else if pid_l.contains("lite") {
                Some("Lite".to_string())
            } else if pid_l.contains("start") {
                Some("Start".to_string())
            } else {
                None
            };
            let mut items: Vec<QuotaItem> = balances
                .iter()
                .filter(|b| {
                    (!b.user_plan_id.is_empty() && b.user_plan_id == user_plan_id)
                        || (b.user_plan_id.is_empty() && b.plan_id == plan_id)
                })
                .map(|b| b.item.clone())
                .collect();
            if items.is_empty() {
                if let Some(ents) = p.get("entitlements").and_then(|v| v.as_array()) {
                    for e in ents {
                        let show = e.get("show_name").and_then(|v| v.as_str()).unwrap_or("");
                        if show.is_empty() {
                            continue;
                        }
                        let unit = e.get("unit_type").and_then(|v| v.as_str()).unwrap_or("token");
                        items.push(QuotaItem {
                            name: show.to_string(),
                            unit: unit.to_string(),
                            unit_code: unit.to_string(),
                            kind: "grant".to_string(),
                            total: e.get("grant_units").and_then(|v| num(v)),
                            used: None,
                            remaining: None,
                            percent_used: None,
                            window: None,
                            period_end: String::new(),
                            reset: String::new(),
                        });
                    }
                }
            }
            out.plans.push(QuotaPlan {
                tier: tier.clone(),
                tier_code: tier.as_ref().map(|t| t.to_lowercase()),
                name: p
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Plan")
                    .to_string(),
                expire: if ends_at > 0 { Some(fmt_secs(ends_at)) } else { None },
                pid: if plan_id.is_empty() { None } else { Some(plan_id) },
                items,
            });
        }
    }

    // top-level items: all balances of live plans (single-plan render path)
    out.items = balances
        .iter()
        .filter(|b| {
            b.user_plan_id.is_empty() || !expired_user_plans.iter().any(|e| *e == b.user_plan_id)
        })
        .map(|b| b.item.clone())
        .collect();
    out.is_empty = out.plans.is_empty() && !saw_any;
    if let Some(first) = out.plans.first().cloned() {
        out.plan_tier = first.tier.clone();
        out.plan_expire = first.expire.clone();
    }
    Ok(out)
}

/// Quota query: every family authenticates with the zcodejwttoken credential.
pub fn query(credentials: &Value, lang: Lang) -> Result<QuotaData, String> {
    let jwt = credentials
        .get("zcodejwttoken")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !jwt.is_empty() {
        return zai_balance(jwt, lang);
    }
    Err(crate::i18n::t(lang, "err.quota.no_token", &[]))
}
