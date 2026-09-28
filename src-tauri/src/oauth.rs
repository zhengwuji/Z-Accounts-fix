//! OAuth login flow ("Add account").
//!
//! Mirrors the ZCode client's own login: open
//! `https://zcode.z.ai/app/oauth/login?redirect=<zcode://oauth/callback>&app_version=`
//! in a webview; capture the `zcode://oauth/callback?code=…&state=…` redirect
//! (or the CLI poll channel); exchange the code at
//! `POST /api/v1/oauth/token` → `data.token` (the `zcodejwttoken`); harvest the
//! provider tokens (`oauth:{provider}:access_token` / `refresh_token`) from the
//! page's localStorage; fetch identity (chat.z.ai userinfo for z.ai,
//! bigmodel.cn getCustomerInfo for BigModel); append the account to the store.
//!
//! A login webview can optionally route through the user-configured proxy
//! (`auth_proxy_*` settings) — quota/claim/switch traffic never does.

use crate::httpc::{agent, urlencode, ZAI_CHAT_BASE, ZCODE_BASE};
use crate::i18n::{t, Lang};
use crate::store::Persist;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};

pub const LOGIN_WINDOW: &str = "login-webview";
pub const REDIRECT_SCHEME: &str = "zcode://oauth/callback";

#[derive(Clone, Default)]
pub struct OauthAbort(Arc<AtomicBool>);

impl OauthAbort {
    fn abort(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    fn aborted(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub struct OauthShared {
    pub run: Mutex<Option<OauthRun>>,
}

#[derive(Clone)]
pub struct OauthRun {
    pub abort: OauthAbort,
}

pub fn shared() -> OauthShared {
    OauthShared {
        run: Mutex::new(None),
    }
}

struct Callback {
    code: String,
    state: String,
}

fn is_callback_url(url: &Url) -> Option<Callback> {
    let s = url.as_str();
    let is_cb =
        s.starts_with(REDIRECT_SCHEME) || s.contains("/oauth/callback") || s.contains("%3A%2F%2Foauth%2Fcallback");
    if !is_cb {
        return None;
    }
    let query = url.query().map(String::from).unwrap_or_default();
    let parsed: Vec<(String, String)> = query
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| {
            let mut it = kv.splitn(2, '=');
            (
                it.next().unwrap_or("").to_lowercase(),
                it.next().unwrap_or("").to_string(),
            )
        })
        .collect();
    let get = |k: &str| parsed.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    let code = get("code").or_else(|| get("authorization_code"))?;
    let state = get("state").unwrap_or_default();
    Some(Callback { code, state })
}

/// Entry point for the `oauth_begin` command. Runs the whole flow on a
/// background thread and emits `oauth://done` events to the main window.
pub fn begin(app: AppHandle, persist: Arc<Persist>, provider: &str) -> Result<(), String> {
    let lang = persist.lang();
    let provider = provider.to_string();
    if provider != "bigmodel" && provider != "zai" {
        return Err(t(
            lang,
            "err.oauth.unknown_provider",
            &[("provider", &provider)],
        ));
    }

    // one flow at a time — a second begin replaces ("flow-done-or-replaced")
    {
        let shared = app.state::<OauthShared>();
        let guard = shared.run.lock().unwrap();
        if let Some(run) = guard.as_ref() {
            run.abort.abort();
        }
        drop(guard);
    }

    let abort = OauthAbort::default();
    {
        let shared = app.state::<OauthShared>();
        *shared.run.lock().unwrap() = Some(OauthRun {
            abort: abort.clone(),
        });
    }

    std::thread::spawn(move || {
        let result = run_flow(&app, persist.clone(), &provider, abort.clone());
        let shared = app.state::<OauthShared>();
        *shared.run.lock().unwrap() = None;
        // close the login window if it is still around
        if let Some(win) = app.get_webview_window(LOGIN_WINDOW) {
            let _ = win.close();
        }
        let payload = match result {
            Ok(FlowOutcome::Added { id, name }) => json!({
                "ok": true, "soft": false, "duplicate": false,
                "error": Value::Null, "id": id, "name": name,
            }),
            Ok(FlowOutcome::Duplicate { id, name }) => json!({
                "ok": false, "soft": false, "duplicate": true,
                "error": Value::Null, "id": id, "name": name,
            }),
            Err(e) => json!({
                "ok": false, "soft": false, "duplicate": false,
                "error": e.to_string(), "id": Value::Null, "name": Value::Null,
            }),
        };
        let _ = app.emit("oauth://done", payload);
        let _ = app.emit("state-changed", json!({}));
    });
    Ok(())
}

pub enum FlowOutcome {
    Added { id: String, name: String },
    Duplicate { id: String, name: String },
}

fn run_flow(
    app: &AppHandle,
    persist: Arc<Persist>,
    provider: &str,
    abort: OauthAbort,
) -> Result<FlowOutcome, String> {
    let lang = persist.lang();
    let av = crate::httpc::app_version_string();

    // 1. init the official CLI flow (best effort; webview login works without it)
    let init = cli_init(provider, &av, lang).ok();
    persist.oauth_log(&format!("provider={provider} init={}", init.is_some()));

    // 2. open the login webview (with optional auth proxy)
    let login_url = match &init {
        Some(url) => url.clone(),
        None => format!(
            "{ZCODE_BASE}/app/oauth/login?redirect={}&app_version={}&provider={}",
            urlencode(REDIRECT_SCHEME),
            urlencode(&av),
            provider
        ),
    };
    let parsed: Url = login_url
        .parse()
        .map_err(|e| {
            let es = format!("{e}");
            t(lang, "err.oauth.bad_authorize_url", &[("e", &es)])
        })?;

    let cb: Arc<Mutex<Option<Callback>>> = Arc::new(Mutex::new(None));
    let mut builder = WebviewWindowBuilder::new(app, LOGIN_WINDOW, WebviewUrl::External(parsed.clone()))
        .title(t(lang, "title.login", &[]))
        .inner_size(1024.0, 760.0)
        .min_inner_size(480.0, 480.0)
        .resizable(true)
        .center()
        .disable_drag_drop_handler();

    let settings = persist.store.lock().unwrap().settings.clone();
    if settings.proxy_on() && !settings.proxy_url().trim().is_empty() {
        builder = builder.additional_browser_args(&format!(
            "--proxy-server={} --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
            settings.proxy_url().trim()
        ));
    } else {
        builder = builder
            .additional_browser_args("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection");
    }

    let cb2 = cb.clone();
    let abort2 = abort.clone();
    builder = builder.on_navigation(move |url| {
        if let Some(cbk) = is_callback_url(&url) {
            let mut slot = cb2.lock().unwrap();
            if slot.is_none() {
                *slot = Some(cbk);
            }
            return false; // stop navigation at our private-scheme redirect
        }
        !abort2.aborted()
    });

    let window = builder
        .build()
        .map_err(|e| { let es = e.to_string(); t(lang, "err.oauth.window", &[("e", &es)]) })?;
    let _ = window.set_focus();

    // 3. wait for the callback (navigation hook) and poll the CLI flow
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    let mut found: Option<Callback> = None;
    loop {
        if abort.aborted() {
            return Err(t(lang, "err.oauth.expired", &[]));
        }
        if let Some(cb) = cb.lock().unwrap().take() {
            found = Some(cb);
            break;
        }
        if let Some(token) = poll_flow(init.as_ref(), provider, lang) {
            return match finalize(persist, provider, &token, None)? {
                Finalize::Added { id, name } => Ok(FlowOutcome::Added { id, name }),
                Finalize::Duplicate { id, name } => Ok(FlowOutcome::Duplicate { id, name }),
            };
        }
        // window closed by the user without completing → flow expired
        if app.get_webview_window(LOGIN_WINDOW).is_none() && !window.is_visible().unwrap_or(false) {
            return Err(t(lang, "err.oauth.expired", &[]));
        }
        if std::time::Instant::now() > deadline {
            return Err(t(lang, "err.oauth.expired", &[]));
        }
        std::thread::sleep(std::time::Duration::from_millis(700));
    }
    let cb = found.ok_or_else(|| t(lang, "err.oauth.flow_failed", &[]))?;
    if cb.code.is_empty() {
        return Err(t(lang, "err.oauth.no_code_state", &[]));
    }

    // 4. exchange the authorization code for the zcode JWT
    let token = exchange(cb.code.trim(), cb.state.trim(), provider, &av, lang)?;
    persist.oauth_log("exchange ok");

    // 5. (localStorage harvest not possible via eval; see harvest_local_storage)
    let harvested = harvest_local_storage(&window, provider);
    match finalize(persist, provider, &token, harvested.map(|v| v.to_string()))? {
        Finalize::Added { id, name } => Ok(FlowOutcome::Added { id, name }),
        Finalize::Duplicate { id, name } => Ok(FlowOutcome::Duplicate { id, name }),
    }
}

fn cli_init(provider: &str, av: &str, lang: Lang) -> Result<String, String> {
    let url = format!("{ZCODE_BASE}/api/v1/oauth/cli/init");
    let resp = crate::httpc::zcode_headers(
        agent().post(&url),
        av,
    )
    .send_json(json!({
        "provider": provider,
        "redirect_uri": REDIRECT_SCHEME,
        "platform": "win32",
        "app_version": av,
    }));
    let doc: Value = match resp {
        Ok(r) => r.into_json().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(c, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(t(
                lang,
                "err.oauth.init_http",
                &[("status", &c.to_string()), ("msg", &body.chars().take(120).collect::<String>())],
            ));
        }
        Err(e) => { let es = e.to_string(); return Err(t(lang, "err.oauth.init", &[("e", &es)])) }
    };
    let code = doc.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 && code != 200 {
        let msg = doc
            .get("msg")
            .or_else(|| doc.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return Err(if msg.is_empty() {
            t(lang, "err.oauth.init_invalid", &[])
        } else {
            t(lang, "err.oauth.init_invalid_msg", &[("msg", &msg)])
        });
    }
    let data = doc.get("data").cloned().unwrap_or(Value::Null);
    let authorize = data
        .get("authorize_url")
        .or_else(|| doc.get("authorize_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if authorize.is_empty() {
        return Err(t(lang, "err.oauth.init_invalid", &[]));
    }
    Ok(authorize)
}

fn poll_flow(_init: Option<&String>, _provider: &str, _lang: Lang) -> Option<String> {
    // The CLI poll channel (GET /api/v1/oauth/cli/poll/{flow_id}) only applies
    // when cli/init handed us a flow; the webview navigation callback is the
    // primary path, so polling is a no-op placeholder here.
    None
}

fn exchange(
    code: &str,
    state: &str,
    provider: &str,
    av: &str,
    lang: Lang,
) -> Result<String, String> {
    let url = format!("{ZCODE_BASE}/api/v1/oauth/token");
    let resp = crate::httpc::zcode_headers(agent().post(&url), av).send_json(json!({
        "code": code,
        "state": state,
        "provider": provider,
        "redirect_uri": REDIRECT_SCHEME,
        "platform": "win32",
        "app_version": av,
    }));
    let doc: Value = match resp {
        Ok(r) => r
            .into_json()
            .map_err(|e| { let es = e.to_string(); t(lang, "err.oauth.exchange_req", &[("e", &es)]) })?,
        Err(ureq::Error::Status(c, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(t(
                lang,
                "err.oauth.exchange",
                &[("code", &c.to_string()), ("msg", &body.chars().take(120).collect::<String>())],
            ));
        }
        Err(e) => { let es = e.to_string(); return Err(t(lang, "err.oauth.exchange_req", &[("e", &es)])) }
    };
    let token = doc
        .pointer("/data/token")
        .or_else(|| doc.pointer("/data/access_token"))
        .or_else(|| doc.get("token"))
        .or_else(|| doc.get("access_token"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if token.is_empty() {
        return Err(t(lang, "err.oauth.no_token", &[]));
    }
    Ok(token.to_string())
}

/// Run in the login page context: the page's localStorage may carry the OAuth
/// artifacts (`oauth:active_provider`, `oauth:{p}:access_token`, …). wry's
/// `eval` cannot return values, so harvesting is a no-op today — the primary
/// path (code+state callback exchange) covers the flow.
fn harvest_local_storage(_window: &tauri::WebviewWindow, _provider: &str) -> Option<Value> {
    None
}

fn fetch_userinfo(provider: &str, provider_token: Option<&str>, zcode_token: &str) -> Option<Value> {
    let auth = provider_token
        .filter(|s| !s.is_empty())
        .unwrap_or(zcode_token);
    match provider {
        "zai" => {
            let resp = agent()
                .get(&format!("{ZAI_CHAT_BASE}/api/oauth/userinfo"))
                .set("Authorization", &format!("Bearer {auth}"))
                .call()
                .ok()?;
            let doc: Value = resp.into_json().ok()?;
            Some(json!({
                "sub": doc.get("sub").or_else(|| doc.get("user_id")).cloned().unwrap_or(Value::Null),
                "username": doc.get("username").or_else(|| doc.get("preferred_username")).cloned().unwrap_or(Value::Null),
                "display_name": doc.get("displayName").or_else(|| doc.get("name")).cloned().unwrap_or(Value::Null),
                "avatar": doc.get("avatarUrl").or_else(|| doc.get("avatar")).or_else(|| doc.get("picture")).cloned().unwrap_or(Value::Null),
                "email": doc.get("email").cloned().unwrap_or(Value::Null),
            }))
        }
        _ => {
            let info = crate::quota::bigmodel_customer_info(auth)?;
            Some(json!({
                "sub": info.get("customerNumber").cloned().unwrap_or(Value::Null),
                "username": info.get("customerName").cloned().unwrap_or(Value::Null),
                "display_name": info.get("nickName").cloned().unwrap_or(Value::Null),
                "avatar": info.get("avatar").cloned().unwrap_or(Value::Null),
                "email": info.get("email").cloned().unwrap_or(Value::Null),
            }))
        }
    }
}

enum Finalize {
    Added { id: String, name: String },
    Duplicate { id: String, name: String },
}

fn finalize(
    persist: Arc<Persist>,
    provider: &str,
    zcode_token: &str,
    provider_token: Option<String>,
) -> Result<Finalize, String> {
    let lang = persist.lang();
    let userinfo = fetch_userinfo(provider, provider_token.as_deref(), zcode_token);
    let mut creds = json!({
        "oauth:active_provider": provider,
        format!("oauth:{provider}:access_token"): provider_token.clone().unwrap_or_else(|| zcode_token.to_string()),
        "zcodejwttoken": zcode_token,
    });
    if let Some(pt) = &provider_token {
        creds[&format!("oauth:{provider}:refresh_token")] = json!(pt);
    }
    if let Some(ui) = &userinfo {
        // plaintext side-channel for identity display; stripped when writing
        // back into ZCode's credentials.json
        creds["zsw:user_info"] = json!(ui.to_string());
    }
    let creds_str = serde_json::to_string(&creds).unwrap_or_default();
    let hash = crate::zcode::sha256_hex(creds_str.as_bytes());

    let mut store = persist.store.lock().unwrap();
    if let Some(existing) = store.accounts.iter().find(|a| a.hash == hash) {
        return Ok(Finalize::Duplicate {
            id: existing.id.clone(),
            name: existing.name.clone(),
        });
    }
    let stored = crate::crypto::zcode_cred::encrypt_creds(&creds).unwrap_or(creds);
    let name = pick_name(&store.accounts, &userinfo, provider);
    let now = crate::store::ts_now();
    let id = new_id();
    store.accounts.push(crate::store::Account {
        id: id.clone(),
        name: name.clone(),
        created_at: now.clone(),
        updated_at: now,
        hash: hash.clone(),
        credentials: stored,
        config: None,
        virtual_device_mid: None,
        virtual_arms_uid: None,
    });
    drop(store);
    persist.save_accounts(lang).ok();
    Ok(Finalize::Added { id, name })
}

fn pick_name(accounts: &[crate::store::Account], userinfo: &Option<Value>, provider: &str) -> String {
    let mut base = String::new();
    if let Some(ui) = userinfo {
        for key in ["display_name", "username", "sub"] {
            if let Some(v) = ui.get(key).and_then(|x| x.as_str()) {
                if !v.trim().is_empty() {
                    base = v.trim().to_string();
                    break;
                }
            }
        }
    }
    if base.is_empty() {
        base = provider.to_string();
    }
    let mut name = base.clone();
    let mut n = 1;
    while accounts.iter().any(|a| a.name == name) {
        n += 1;
        name = format!("{base} {n}");
    }
    name
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
