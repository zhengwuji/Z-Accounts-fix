//! All Tauri IPC commands. Names/args exactly match the frontend's invoke calls.

use crate::i18n::{t, Lang};
use crate::store::{Account, Persist, BUNDLE_FORMAT};
use crate::zcode;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

type PersistState<'a> = State<'a, Arc<Persist>>;

fn lang_of(persist: &Persist) -> Lang {
    persist.lang()
}

/// Emit an event to any live windows (no-op when the app handle isn't set,
/// i.e. during headless CLI runs).
pub fn emit_global(event: &str, payload: Value) {
    if let Some(app) = crate::app_handle() {
        let _ = app.emit(event, payload);
    }
}

// ------------------------------------------------------------- identity

/// Derive an identity view from a credentials JSON blob.
pub fn identity_of(creds: &Value) -> Value {
    let mut username = String::new();
    let mut display = String::new();
    let mut email = String::new();
    let mut user_id = String::new();
    let mut avatar = String::new();

    if let Some(s) = creds.get("zsw:user_info").and_then(|v| v.as_str()) {
        if let Ok(ui) = serde_json::from_str::<Value>(s) {
            username = ui.get("username").and_then(|x| x.as_str()).unwrap_or("").into();
            display = ui.get("display_name").and_then(|x| x.as_str()).unwrap_or("").into();
            email = ui.get("email").and_then(|x| x.as_str()).unwrap_or("").into();
            user_id = ui.get("sub").and_then(|x| x.as_str().map(String::from).or_else(|| x.as_i64().map(|n| n.to_string())).as_deref().map(String::from)).unwrap_or_default();
            avatar = ui.get("avatar").and_then(|x| x.as_str()).unwrap_or("").into();
        }
    }
    // zcodejwttoken claims → user_id fallback
    if user_id.is_empty() {
        if let Some(jwt) = creds.get("zcodejwttoken").and_then(|v| v.as_str()) {
            if let Some(sub) = jwt_claim(jwt, "sub").or_else(|| jwt_claim(jwt, "user_id")) {
                user_id = sub;
            }
            if username.is_empty() {
                username = jwt_claim(jwt, "username").unwrap_or_default();
            }
        }
    }
    let _has_info = creds.get("zcodejwttoken").is_some()
        && (!user_id.is_empty() || !username.is_empty() || creds.get("zsw:user_info").is_some());
    json!({
        "username": username,
        "display_name": if display.is_empty() { username.clone() } else { display },
        "email": email,
        "user_id": user_id,
        "avatar": avatar,
    })
}

fn jwt_claim(jwt: &str, claim: &str) -> Option<String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = B64.decode(parts[1]).ok()?;
    let doc: Value = serde_json::from_slice(&payload).ok()?;
    doc.get(claim)
        .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_i64().map(|n| n.to_string())))
}

fn strip_tool_keys(creds: &mut Value) {
    if let Some(obj) = creds.as_object_mut() {
        let keys: Vec<String> = obj
            .keys()
            .filter(|k| k.starts_with("zsw:"))
            .cloned()
            .collect();
        for k in keys {
            obj.remove(&k);
        }
    }
}

// ------------------------------------------------------------- get_state

fn account_view(a: &Account, active: bool) -> Value {
    let plain = crate::crypto::zcode_cred::decrypt_creds(&a.credentials);
    let identity = identity_of(&plain);
    json!({
        "id": a.id,
        "name": a.name,
        "is_active": active,
        "has_config": a.config.is_some(),
        "has_user_info": a.credentials.get("zcodejwttoken").is_some(),
        "identity": identity,
        "created_at": a.created_at,
        "updated_at": a.updated_at,
        "hash": a.hash,
    })
}

#[tauri::command]
pub fn get_state_core(persist: &Arc<Persist>) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let g = persist.store.lock().unwrap();
    let live_hash = zcode::credentials_exists()
        .then(|| zcode::read_json_file(&zcode::credentials_path(), lang).ok())
        .flatten()
        .map(|v| zcode::hash_json(&v))
        .unwrap_or_default();
    let creds_file = zcode::credentials_exists();
    let live_logged_in = creds_file && (|| {
        zcode::read_json_file(&zcode::credentials_path(), lang)
            .map(|v| zcode::live_has_credentials(&v))
            .unwrap_or(false)
    })();
    let live_identity = if live_logged_in {
        zcode::read_json_file(&zcode::credentials_path(), lang)
            .map(|v| identity_of(&v))
            .unwrap_or_else(|_| json!(null))
    } else {
        json!(null)
    };
    let active_id = g
        .accounts
        .iter()
        .find(|a| !live_hash.is_empty() && a.hash == live_hash)
        .map(|a| a.id.clone());
    let accounts: Vec<Value> = g
        .accounts
        .iter()
        .map(|a| account_view(a, Some(&a.id) == active_id.as_ref()))
        .collect();
    let exe = zcode::find_zcode_exe(&g.settings.zcode_path().to_string());
    Ok(json!({
        "accounts": accounts,
        "zcode_path": g.settings.zcode_path().to_string(),
        "launch_after_switch": g.settings.launch_after_switch(),
        "close_to_tray": g.settings.close_to_tray(),
        "hot_switch": g.settings.hot_switch(),
        "auth_proxy_on": g.settings.proxy_on(),
        "auth_proxy_url": g.settings.proxy_url().to_string(),
        "language": g.settings.language.clone().unwrap_or_else(|| "zh".into()),
        "theme": g.settings.theme(),
        "auto_claim": g.settings.auto_claim(),
        "auto_switch": g.settings.auto_switch(),
        "zcode_running": zcode::is_running(),
        "live_exists": creds_file,
        "live_logged_in": live_logged_in,
        "live_hash": live_hash,
        "live_identity": live_identity,
        "active_account_id": active_id,
        "zcode_path_ok": exe.is_some(),
        "store_dir": persist.dir.display().to_string(),
    }))
}

#[tauri::command]
pub fn get_state(persist: PersistState) -> Result<Value, String> { get_state_core(persist.inner()) }

#[tauri::command]
pub fn app_version() -> String {
    crate::httpc::app_version_string()
}

#[tauri::command]
pub fn get_recent_model_status() -> Value {
    match zcode::recent_model_status() {
        Some(s) => json!({
            "kind": s.kind, "at": s.at, "model": s.model, "provider": s.provider,
            "status_code": s.status_code, "provider_code": s.provider_code,
            "request_id": s.request_id,
        }),
        None => Value::Null,
    }
}

#[tauri::command]
pub fn set_language_core(persist: &Arc<Persist>, lang: String) -> Result<(), String> {
    let l = lang_of(&persist);
    if lang != "zh" && lang != "en" {
        return Err(t(l, "err.lang.unknown", &[("lang", &lang)]));
    }
    persist.store.lock().unwrap().settings.language = Some(lang);
    persist.save_settings(l)
}

#[tauri::command]
pub fn set_language(persist: PersistState, lang: String) -> Result<(), String> { set_language_core(persist.inner(), lang) }

#[tauri::command]
pub fn set_theme_core(persist: &Arc<Persist>, theme: String) -> Result<(), String> {
    let l = lang_of(&persist);
    if theme != "dark" && theme != "light" {
        return Err(t(l, "err.lang.unknown", &[("lang", &theme)]));
    }
    persist.store.lock().unwrap().settings.theme = Some(theme);
    persist.save_settings(l)
}

#[tauri::command]
pub fn set_theme(persist: PersistState, theme: String) -> Result<(), String> { set_theme_core(persist.inner(), theme) }

#[tauri::command]
pub fn set_behavior_core(
    persist: &Arc<Persist>,
    launch_after_switch: Option<bool>,
    close_to_tray: Option<bool>,
    hot_switch: Option<bool>,
    auto_claim: Option<bool>,
    auto_switch: Option<bool>,
) -> Result<(), String> {
    let l = lang_of(&persist);
    {
        let mut g = persist.store.lock().unwrap();
        if let Some(v) = launch_after_switch {
            g.settings.launch_after_switch = Some(v);
        }
        if let Some(v) = close_to_tray {
            g.settings.close_to_tray = Some(v);
        }
        if let Some(v) = hot_switch {
            g.settings.hot_switch = Some(v);
        }
        if let Some(v) = auto_claim {
            g.settings.auto_claim = Some(v);
        }
        if let Some(v) = auto_switch {
            g.settings.auto_switch = Some(v);
        }
    }
    persist.save_settings(l)?;
    let _ = persist.save_accounts(l);
    Ok(())
}

#[tauri::command]
pub fn set_behavior(
    persist: PersistState,
    launch_after_switch: Option<bool>,
    close_to_tray: Option<bool>,
    hot_switch: Option<bool>,
    auto_claim: Option<bool>,
    auto_switch: Option<bool>,
) -> Result<(), String> { set_behavior_core(persist.inner(), launch_after_switch, close_to_tray, hot_switch, auto_claim, auto_switch) }

#[tauri::command]
pub fn set_auth_proxy_core(
    persist: &Arc<Persist>,
    on: bool,
    url: Option<String>,
) -> Result<(), String> {
    let l = lang_of(&persist);
    let trimmed = url.unwrap_or_default().trim().to_string();
    if on {
        if trimmed.is_empty() {
            return Err(t(l, "err.proxy.need_url", &[]));
        }
        validate_proxy(&trimmed, l)?;
    }
    {
        let mut g = persist.store.lock().unwrap();
        g.settings.auth_proxy_on = Some(on);
        if !trimmed.is_empty() {
            g.settings.auth_proxy_url = Some(trimmed);
        }
    }
    persist.save_settings(l)
}

#[tauri::command]
pub fn set_auth_proxy(
    persist: PersistState,
    on: bool,
    url: Option<String>,
) -> Result<(), String> { set_auth_proxy_core(persist.inner(), on, url) }

pub fn validate_proxy(url: &str, lang: Lang) -> Result<(), String> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| t(lang, "err.proxy.scheme", &[]))?;
    if scheme != "http" && scheme != "socks5" {
        return Err(t(lang, "err.proxy.scheme", &[]));
    }
    if rest.contains('@') {
        return Err(t(lang, "err.proxy.no_auth", &[]));
    }
    if rest.contains('/') {
        return Err(t(lang, "err.proxy.no_path", &[]));
    }
    let (host, port) = rest
        .rsplit_once(':')
        .ok_or_else(|| t(lang, "err.proxy.need_port", &[]))?;
    if host.is_empty() {
        return Err(t(lang, "err.proxy.empty_host", &[]));
    }
    let port_num: u16 = port
        .parse()
        .map_err(|_| t(lang, "err.proxy.port_nan", &[("port", port)]))?;
    if port_num == 0 {
        return Err(t(lang, "err.proxy.port_range", &[("port", &port_num.to_string())]));
    }
    if host.split('.').any(|seg| seg.is_empty()) && host != "localhost" {
        return Err(t(lang, "err.proxy.bad_host", &[]));
    }
    Ok(())
}

// ------------------------------------------------------- capture/switch

#[tauri::command]
pub fn capture_current_core(persist: &Arc<Persist>, name: Option<String>) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let path = zcode::credentials_path();
    if !path.is_file() {
        return Err(t(lang, "err.live.no_creds_file", &[]));
    }
    let creds = zcode::read_json_file(&path, lang)?;
    if !creds.is_object() {
        return Err(t(lang, "err.store.not_object", &[]));
    }
    if !zcode::live_has_credentials(&creds) {
        return Err(t(lang, "err.live.no_credentials", &[]));
    }
    let hash = zcode::hash_json(&creds);
    {
        let g = persist.store.lock().unwrap();
        if let Some(existing) = g.accounts.iter().find(|a| a.hash == hash) {
            return Err(t(
                lang,
                "err.live.dup_saved",
                &[("name", &existing.name)],
            ));
        }
    }
    let config = if zcode::config_path().is_file() {
        zcode::read_json_file(&zcode::config_path(), lang).ok()
    } else {
        None
    };
    let name = match name.filter(|n| !n.trim().is_empty()) {
        Some(n) => {
            crate::store::validate_name(lang, &n)?;
            n.trim().to_string()
        }
        None => {
            let ident = identity_of(&creds);
            let mut base = String::new();
            for key in ["display_name", "username", "user_id"] {
                if let Some(v) = ident.get(key).and_then(|x| x.as_str()) {
                    if !v.trim().is_empty() && v != "null" {
                        base = v.trim().to_string();
                        break;
                    }
                }
            }
            if base.is_empty() {
                base = format!("Account {}", chrono::Local::now().format("%m%d-%H%M%S"));
            }
            let mut final_name = base.clone();
            let mut n = 1;
            while persist
                .store
                .lock()
                .unwrap()
                .accounts
                .iter()
                .any(|a| a.name == final_name)
            {
                n += 1;
                final_name = format!("{base} {n}");
            }
            final_name
        }
    };
    {
        let mut g = persist.store.lock().unwrap();
        if g.accounts.iter().any(|a| a.name == name) {
            let other = g
                .accounts
                .iter()
                .find(|a| a.name == name)
                .map(|a| a.name.clone())
                .unwrap_or_default();
            return Err(crate::store::name_taken(lang, &name, &other));
        }
        let now = crate::store::ts_now();
        let stored = crate::crypto::zcode_cred::encrypt_creds(&creds).unwrap_or(creds);
        g.accounts.push(Account {
            id: crate::oauth::new_id(),
            name: name.clone(),
            created_at: now.clone(),
            updated_at: now,
            hash,
            credentials: stored,
            config,
            virtual_device_mid: zcode::current_device_mid(),
            virtual_arms_uid: zcode::current_arms_uid(),
        });
    }
    persist.save_accounts(lang)?;
    Ok(json!({ "name": name }))
}

#[tauri::command]
pub fn capture_current(persist: PersistState, name: Option<String>) -> Result<Value, String> { capture_current_core(persist.inner(), name) }

#[tauri::command]
pub fn rename_account_core(persist: &Arc<Persist>, id: String, name: String) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let name = name.trim().to_string();
    crate::store::validate_name(lang, &name)?;
    let mut g = persist.store.lock().unwrap();
    if g.accounts.iter().any(|a| a.name == name && a.id != id) {
        let other = g
            .accounts
            .iter()
            .find(|a| a.name == name && a.id != id)
            .map(|a| a.name.clone())
            .unwrap_or_default();
        return Err(crate::store::name_taken(lang, &name, &other));
    }
    let acc = g
        .accounts
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
    acc.name = name.clone();
    acc.updated_at = crate::store::ts_now();
    drop(g);
    persist.save_accounts(lang)?;
    Ok(json!({ "name": name }))
}

#[tauri::command]
pub fn rename_account(persist: PersistState, id: String, name: String) -> Result<Value, String> { rename_account_core(persist.inner(), id, name) }

#[tauri::command]
pub fn delete_account_core(persist: &Arc<Persist>, id: String) -> Result<(), String> {
    let lang = lang_of(&persist);
    let mut g = persist.store.lock().unwrap();
    let before = g.accounts.len();
    g.accounts.retain(|a| a.id != id);
    if g.accounts.len() == before {
        return Err(t(lang, "err.store.no_account", &[]));
    }
    drop(g);
    persist.save_accounts(lang)
}

#[tauri::command]
pub fn delete_account(persist: PersistState, id: String) -> Result<(), String> { delete_account_core(persist.inner(), id) }

#[tauri::command]
pub fn update_account_from_live_core(persist: &Arc<Persist>, id: String) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let path = zcode::credentials_path();
    if !path.is_file() {
        return Err(t(lang, "err.live.no_file", &[]));
    }
    let creds = zcode::read_json_file(&path, lang)?;
    if !zcode::live_has_credentials(&creds) {
        return Err(t(lang, "err.live.logged_out", &[]));
    }
    let hash = zcode::hash_json(&creds);
    let config = if zcode::config_path().is_file() {
        zcode::read_json_file(&zcode::config_path(), lang).ok()
    } else {
        None
    };
    {
        let mut g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        if acc.hash != hash {
            return Err(t(lang, "err.live.same", &[("name", &acc.name)]));
        }
        acc.credentials =
            crate::crypto::zcode_cred::encrypt_creds(&creds).unwrap_or(creds);
        if config.is_some() {
            acc.config = config;
        }
        acc.hash = hash;
        acc.updated_at = crate::store::ts_now();
    }
    persist.save_accounts(lang)?;
    let g = persist.store.lock().unwrap();
    let name = g
        .accounts
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.name.clone())
        .unwrap_or_default();
    Ok(json!({ "name": name }))
}

#[tauri::command]
pub fn update_account_from_live(persist: PersistState, id: String) -> Result<Value, String> { update_account_from_live_core(persist.inner(), id) }

#[tauri::command]
pub fn switch_to_core(
    persist: &Arc<Persist>,
    id: String,
    force: bool,
    restart: bool,
) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let (mut account, hot_switch, launch_after) = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?
            .clone();
        (acc, g.settings.hot_switch(), g.settings.launch_after_switch())
    };
    // snapshots are stored with enc:v1-encrypted values; ZCode needs plaintext
    account.credentials = crate::crypto::zcode_cred::decrypt_creds(&account.credentials);
    let launch_exe = {
        let g = persist.store.lock().unwrap();
        zcode::find_zcode_exe(&g.settings.zcode_path().to_string())
    };

    let running = zcode::is_running();
    let live_hash = if zcode::credentials_exists() {
        zcode::read_json_file(&zcode::credentials_path(), lang)
            .map(|v| zcode::hash_json(&v))
            .unwrap_or_default()
    } else {
        String::new()
    };

    // already the active login?
    if !live_hash.is_empty() && live_hash == account.hash {
        let mut launched = false;
        let mut launch_error = None;
        if (restart || launch_after) && !running {
            if let Some(exe) = &launch_exe {
                match zcode::launch(exe) {
                    Ok(()) => launched = true,
                    Err(e) => launch_error = Some(t(lang, "err.zcode.launch", &[("e", &e)])),
                }
            }
        }
        return Ok(json!({
            "already_active": true, "hot": false, "killed": false,
            "preserved_as": Value::Null, "launched": launched,
            "launch_error": launch_error, "config_stale": false,
            "name": account.name,
        }));
    }

    // preserve the current (unsaved) login first
    let mut preserved_as = None;
    if live_hash.is_empty() == false {
        let live_logged = zcode::credentials_exists()
            && zcode::read_json_file(&zcode::credentials_path(), lang)
                .map(|v| zcode::live_has_credentials(&v))
                .unwrap_or(false);
        if live_logged {
            let known = persist
                .store
                .lock()
                .unwrap()
                .accounts
                .iter()
                .any(|a| a.hash == live_hash);
            if !known {
                match capture_current_inner(persist, None) {
                    Ok(name) => preserved_as = Some(name),
                    Err(_) => {}
                }
            }
        }
    }

    let mut hot = false;
    let mut killed = false;
    if running {
        if hot_switch {
            hot = true;
        } else if force {
            zcode::kill(lang)?;
            killed = true;
        } else {
            return Err(t(lang, "err.switch.running", &[]));
        }
    }

    // write credentials.json
    let mut creds = account.credentials.clone();
    strip_tool_keys(&mut creds);
    let creds_str = serde_json::to_string_pretty(&creds)
        .map_err(|e| { let es = e.to_string(); t(lang, "err.serialize", &[("e", &es)]) })?;
    let creds_path = zcode::credentials_path();
    if hot {
        zcode::hot_write(&creds_path, creds_str.as_bytes(), lang)?;
    } else {
        fs::write(&creds_path, creds_str.as_bytes())
            .map_err(|e| { let es = e.to_string(); t(lang, "err.write_config", &[("e", &es)]) })?;
    }

    // write config.json when we have a snapshot
    let mut config_stale = false;
    if let Some(cfg) = &account.config {
        let cfg_str = serde_json::to_string_pretty(cfg)
            .map_err(|e| { let es = e.to_string(); t(lang, "err.serialize", &[("e", &es)]) })?;
        let cfg_path = zcode::config_path();
        if hot {
            zcode::hot_write(&cfg_path, cfg_str.as_bytes(), lang)?;
        } else {
            fs::write(&cfg_path, cfg_str.as_bytes())
                .map_err(|e| { let es = e.to_string(); t(lang, "err.write_config", &[("e", &es)]) })?;
        }
    } else {
        config_stale = true; // keep the existing config.json
    }

    // device identity virtualization
    {
        let mut mid = account.virtual_device_mid.clone();
        let mut arms = account.virtual_arms_uid.clone();
        zcode::apply_device_identity(&mut mid, &mut arms, lang)?;
        let mut g = persist.store.lock().unwrap();
        if let Some(acc) = g.accounts.iter_mut().find(|a| a.id == id) {
            acc.virtual_device_mid = mid;
            acc.virtual_arms_uid = arms;
            acc.updated_at = crate::store::ts_now();
        }
        drop(g);
        persist.save_accounts(lang)?;
    }

    // align provider family domain
    zcode::align_family_domain(&creds);

    // launch ZCode when asked
    let want_launch = if hot { false } else { restart || launch_after };
    let mut launched = false;
    let mut launch_error = None;
    if want_launch {
        if let Some(exe) = &launch_exe {
            match zcode::launch(exe) {
                Ok(()) => launched = true,
                Err(e) => launch_error = Some(t(lang, "err.zcode.launch", &[("e", &e)])),
            }
        } else {
            launch_error = Some(t(
                lang,
                "err.zcode.path_invalid_hint",
                &[("p", &account.name)],
            ));
        }
    }

    emit_global("state-changed", json!({}));
    Ok(json!({
        "already_active": false, "hot": hot, "killed": killed,
        "preserved_as": preserved_as, "launched": launched,
        "launch_error": launch_error, "config_stale": config_stale,
        "name": account.name,
    }))
}

#[tauri::command]
pub fn switch_to(
    persist: PersistState,
    id: String,
    force: bool,
    restart: bool,
) -> Result<Value, String> { switch_to_core(persist.inner(), id, force, restart) }

/// Internal capture used by auto-preserve (no duplicate guard against itself).
fn capture_current_inner(persist: &Arc<Persist>, name: Option<String>) -> Result<String, String> {
    let lang = lang_of(persist);
    let path = zcode::credentials_path();
    if !path.is_file() {
        return Err(t(lang, "err.live.no_creds_file", &[]));
    }
    let creds = zcode::read_json_file(&path, lang)?;
    if !zcode::live_has_credentials(&creds) {
        return Err(t(lang, "err.live.no_credentials", &[]));
    }
    let hash = zcode::hash_json(&creds);
    {
        let g = persist.store.lock().unwrap();
        if let Some(existing) = g.accounts.iter().find(|a| a.hash == hash) {
            return Ok(existing.name.clone());
        }
    }
    let config = if zcode::config_path().is_file() {
        zcode::read_json_file(&zcode::config_path(), lang).ok()
    } else {
        None
    };
    let name = match name {
        Some(n) => n,
        None => zcode::preserve_live_name(),
    };
    let now = crate::store::ts_now();
    let stored = crate::crypto::zcode_cred::encrypt_creds(&creds).unwrap_or(creds);
    {
        let mut g = persist.store.lock().unwrap();
        g.accounts.push(Account {
            id: crate::oauth::new_id(),
            name: name.clone(),
            created_at: now.clone(),
            updated_at: now,
            hash,
            credentials: stored,
            config,
            virtual_device_mid: zcode::current_device_mid(),
            virtual_arms_uid: zcode::current_arms_uid(),
        });
    }
    persist.save_accounts(lang)?;
    Ok(name)
}

// ------------------------------------------------------------- zcode ctl

#[tauri::command]
pub fn launch_zcode_core(persist: &Arc<Persist>) -> Result<(), String> {
    let lang = persist.lang();
    let exe = {
        let g = persist.store.lock().unwrap();
        zcode::find_zcode_exe(&g.settings.zcode_path().to_string())
    };
    let exe = match exe {
        Some(e) => e,
        None => {
            let p = {
                let g = persist.store.lock().unwrap();
                g.settings.zcode_path().to_string().clone()
            };
            let shown = if p.is_empty() {
                zcode::default_exe_path().display().to_string()
            } else {
                p
            };
            return Err(t(lang, "err.zcode.missing", &[("path", &shown)]));
        }
    };
    zcode::launch(&exe).map_err(|e| { let es = e.to_string(); t(lang, "err.zcode.launch", &[("e", &es)]) })
}

#[tauri::command]
pub fn launch_zcode(persist: PersistState) -> Result<(), String> { launch_zcode_core(persist.inner()) }

#[tauri::command]
pub fn kill_zcode_core(persist: &Arc<Persist>) -> Result<(), String> {
    let lang = lang_of(&persist);
    zcode::kill(lang).map(|_| ())
}

#[tauri::command]
pub fn kill_zcode(persist: PersistState) -> Result<(), String> { kill_zcode_core(persist.inner()) }

#[tauri::command]
pub fn pick_zcode_path(app: AppHandle, persist: PersistState) -> Result<Value, String> {
    use tauri_plugin_dialog::DialogExt;
    let lang = lang_of(&persist);
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter(t(lang, "dialog.exe", &[]), &["exe"])
        .pick_file(move |p| {
            let _ = tx.send(p.map(|f| f.to_string()));
        });
    match rx.recv() {
        Ok(Some(path)) => Ok(json!({ "picked": true, "path": path })),
        _ => Ok(json!({ "picked": false, "path": Value::Null })),
    }
}

#[tauri::command]
pub fn set_zcode_path_core(persist: &Arc<Persist>, path: String) -> Result<(), String> {
    let lang = lang_of(&persist);
    let trimmed = path.trim().to_string();
    if !trimmed.is_empty() {
        let p = PathBuf::from(&trimmed);
        if !p.is_file() {
            return Err(t(lang, "err.zcode.path_invalid", &[("p", &trimmed)]));
        }
    }
    persist.store.lock().unwrap().settings.zcode_path = Some(trimmed);
    persist.save_settings(lang)
}

#[tauri::command]
pub fn set_zcode_path(persist: PersistState, path: String) -> Result<(), String> { set_zcode_path_core(persist.inner(), path) }

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    // only allow http(s)
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("invalid url".into());
    }
    let mut cmd = std::process::Command::new("cmd");
    crate::zcode::silent(&mut cmd)
        .args(["/C", "start", "", &url])
        .spawn()
        .map_err(|e| e.to_string())?;
    let _ = &app;
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    use tauri::WebviewUrl;
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into()),
    )
    .title("Z-Accounts 设置")
    .inner_size(720.0, 780.0)
    .min_inner_size(620.0, 560.0)
    .resizable(true)
    .center()
    .additional_browser_args(
        "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --no-proxy-server",
    )
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reveal_main(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        Ok(())
    } else {
        Err(t(Lang::Zh, "err.main.missing", &[]))
    }
}

#[tauri::command]
pub fn autostart_status(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn autostart_set(app: AppHandle, persist: PersistState, enable: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enable {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|e| e.to_string())?;
    let lang = persist.lang();
    let _ = persist.save_settings(lang);
    Ok(enable)
}

// ---------------------------------------------------------------- quota

#[tauri::command]
pub fn get_account_quota_core(persist: &Arc<Persist>, id: String) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let creds = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        crate::crypto::zcode_cred::decrypt_creds(&acc.credentials)
    };
    let data = crate::quota::query(&creds, lang)?;
    Ok(data.to_json(chrono::Utc::now().timestamp_millis()))
}

#[tauri::command]
pub fn get_account_quota(persist: PersistState, id: String) -> Result<Value, String> { get_account_quota_core(persist.inner(), id) }

// ---------------------------------------------------------------- claim

#[tauri::command]
pub fn claim_preview_core(persist: &Arc<Persist>, id: String) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let token = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        crate::crypto::zcode_cred::decrypt_creds(&acc.credentials)
            .get("zcodejwttoken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    if token.is_empty() {
        return Err(t(lang, "err.claim.no_jwt", &[]));
    }
    let plans = crate::claim::preview(&token, lang)?;
    Ok(Value::Array(plans.iter().map(|p| p.to_json()).collect()))
}

#[tauri::command]
pub fn claim_preview(persist: PersistState, id: String) -> Result<Value, String> { claim_preview_core(persist.inner(), id) }

#[tauri::command]
pub fn claim_refresh_core(persist: &Arc<Persist>, id: String) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let token = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        crate::crypto::zcode_cred::decrypt_creds(&acc.credentials)
            .get("zcodejwttoken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    if token.is_empty() {
        return Err(t(lang, "err.claim.no_jwt", &[]));
    }
    // activation report first ("refresh eligibility" = re-activate event flow)
    let activation_error = match crate::claim::report_activation(&token, lang) {
        Ok(()) => None,
        Err(e) => Some(e),
    };
    let plans = crate::claim::preview(&token, lang)?;
    Ok(json!({
        "plans": plans.iter().map(|p| p.to_json()).collect::<Vec<_>>(),
        "activationError": activation_error,
    }))
}

#[tauri::command]
pub fn claim_refresh(persist: PersistState, id: String) -> Result<Value, String> { claim_refresh_core(persist.inner(), id) }

#[tauri::command]
pub fn claim_captcha_config(persist: PersistState) -> Result<Value, String> {
    let lang = lang_of(&persist);
    crate::claim::captcha_config(lang)
}

#[tauri::command]
pub fn claim_start(
    app: AppHandle,
    persist: PersistState,
    id: String,
    plan_id: String,
    auto: Option<bool>,
) -> Result<(), String> {
    let lang = lang_of(&persist);
    let token = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        crate::crypto::zcode_cred::decrypt_creds(&acc.credentials)
            .get("zcodejwttoken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    if token.is_empty() {
        return Err(t(lang, "err.claim.no_jwt", &[]));
    }
    let _ = crate::claim::report_activation(&token, lang);
    let outcome = crate::claim::claim(&token, &plan_id, None, None, lang)?;
    if outcome.ok {
        emit_global(
            "claim://result",
            json!({
                "accountId": id, "ok": true, "planName": plan_id,
                "startsAt": outcome.starts_at, "endsAt": outcome.ends_at,
                "serverTime": outcome.server_time, "message": Value::Null,
                "code": Value::Null, "nextAt": Value::Null,
                "accountName": Value::Null,
            }),
        );
        return Ok(());
    }
    // captcha challenge → open the captcha window
    if outcome.code == Some(3007) || outcome.message.contains("验证码") || auto == Some(true) {
        let _ = crate::captcha_window::open(&app, id.clone(), plan_id.clone());
    }
    emit_global(
        "claim://result",
        json!({
            "accountId": id, "ok": false, "message": outcome.message,
            "code": outcome.code, "nextAt": outcome.next_at,
            "planName": plan_id, "startsAt": outcome.starts_at,
            "endsAt": outcome.ends_at, "serverTime": outcome.server_time,
            "accountName": Value::Null,
        }),
    );
    Ok(())
}

#[tauri::command]
pub fn claim_cancel(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("captcha") {
        let _ = win.close();
    }
    let _ = app.emit("captcha://interactive", json!({}));
    Ok(())
}

#[tauri::command]
pub fn claim_captcha_submit(
    app: AppHandle,
    persist: PersistState,
    param: String,
    region: Option<String>,
) -> Result<(), String> {
    let lang = lang_of(&persist);
    if param.is_empty() {
        return Err(t(lang, "err.claim.no_captcha", &[]));
    }
    let (account_id, plan_id) = crate::captcha_window::take_pending(&app)
        .ok_or_else(|| t(lang, "err.claim.none_pending", &[]))?;
    let token = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        crate::crypto::zcode_cred::decrypt_creds(&acc.credentials)
            .get("zcodejwttoken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    if token.is_empty() {
        return Err(t(lang, "err.claim.no_jwt", &[]));
    }
    let outcome = crate::claim::claim(
        &token,
        &plan_id,
        Some(&param),
        Some(&region.unwrap_or_default()),
        lang,
    )?;
    if let Some(win) = app.get_webview_window("captcha") {
        let _ = win.close();
    }
    emit_global(
        "claim://result",
        json!({
            "accountId": account_id, "ok": outcome.ok,
            "message": if outcome.ok { Value::Null } else { json!(outcome.message) },
            "code": outcome.code, "nextAt": outcome.next_at,
            "planName": plan_id, "startsAt": outcome.starts_at,
            "endsAt": outcome.ends_at, "serverTime": outcome.server_time,
            "accountName": Value::Null,
        }),
    );
    Ok(())
}

// ------------------------------------------------------ export / import

fn bundle_accounts(accounts: &[Account]) -> Value {
    json!({
        "format": BUNDLE_FORMAT,
        "version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "accounts": accounts.iter().map(|a| json!({
            "name": a.name,
            "created_at": a.created_at,
            "updated_at": a.updated_at,
            "hash": a.hash,
            "credentials": a.credentials,
            "config": a.config,
            "virtual_device_mid": a.virtual_device_mid,
            "virtual_arms_uid": a.virtual_arms_uid,
        })).collect::<Vec<_>>(),
    })
}

#[tauri::command]
pub fn export_pick_path(
    app: AppHandle,
    persist: PersistState,
    id: String,
) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let name = {
        let g = persist.store.lock().unwrap();
        let acc = g
            .accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?;
        acc.name.clone()
    };
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    let default_name = format!("{name}.zsb");
    app.dialog()
        .file()
        .add_filter(t(lang, "dialog.zsb", &[]), &["zsb"])
        .set_file_name(&default_name)
        .save_file(move |p| {
            let _ = tx.send(p.map(|f| f.to_string()));
        });
    match rx.recv() {
        Ok(Some(path)) => Ok(json!({ "picked": true, "path": path, "name": name })),
        _ => Ok(json!({ "picked": false, "path": Value::Null, "name": name })),
    }
}

#[tauri::command]
pub fn export_finalize_core(
    persist: &Arc<Persist>,
    path: String,
    id: String,
    password: String,
) -> Result<Value, String> {
    let lang = lang_of(&persist);
    if password.is_empty() {
        return Err(t(lang, "err.cipher.pw_empty", &[]));
    }
    let account = {
        let g = persist.store.lock().unwrap();
        g.accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| t(lang, "err.store.no_account", &[]))?
    };
    let doc = bundle_accounts(std::slice::from_ref(&account));
    write_bundle(&persist, &path, &doc, &password, lang)?;
    Ok(json!({ "path": path, "count": 1 }))
}

#[tauri::command]
pub fn export_finalize(
    persist: PersistState,
    path: String,
    id: String,
    password: String,
) -> Result<Value, String> { export_finalize_core(persist.inner(), path, id, password) }

#[tauri::command]
pub fn export_all_pick_path(app: AppHandle, persist: PersistState) -> Result<Value, String> {
    let lang = lang_of(&persist);
    let count = persist.store.lock().unwrap().accounts.len();
    if count == 0 {
        return Err(t(lang, "err.export.empty", &[]));
    }
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter(t(lang, "dialog.zsb", &[]), &["zsb"])
        .set_file_name("zcode-accounts.zsb")
        .save_file(move |p| {
            let _ = tx.send(p.map(|f| f.to_string()));
        });
    match rx.recv() {
        Ok(Some(path)) => Ok(json!({ "picked": true, "path": path, "count": count })),
        _ => Ok(json!({ "picked": false, "path": Value::Null, "count": count })),
    }
}

#[tauri::command]
pub fn export_all_finalize_core(
    persist: &Arc<Persist>,
    path: String,
    password: String,
) -> Result<Value, String> {
    let lang = lang_of(&persist);
    if password.is_empty() {
        return Err(t(lang, "err.cipher.pw_empty", &[]));
    }
    let accounts = persist.store.lock().unwrap().accounts.clone();
    if accounts.is_empty() {
        return Err(t(lang, "err.export.empty", &[]));
    }
    let doc = bundle_accounts(&accounts);
    write_bundle(&persist, &path, &doc, &password, lang)?;
    Ok(json!({ "path": path, "count": accounts.len() }))
}

#[tauri::command]
pub fn export_all_finalize(
    persist: PersistState,
    path: String,
    password: String,
) -> Result<Value, String> { export_all_finalize_core(persist.inner(), path, password) }

fn write_bundle(
    _persist: &Arc<Persist>,
    path: &str,
    doc: &Value,
    password: &str,
    lang: Lang,
) -> Result<(), String> {
    let sealed = crate::crypto::seal(
        serde_json::to_vec_pretty(doc).map_err(|e| e.to_string())?.as_slice(),
        password,
    )
    .map_err(|e| t(lang, "err.cipher.encrypt", &[("e", &e)]))?;
    let text = serde_json::to_string_pretty(&sealed).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| { let es = e.to_string(); t(lang, "err.write_file", &[("path", path), ("e", &es)]) })
}

/// Pick .zsb files for import → returns sealed docs + parse errors.
#[tauri::command]
pub fn import_pick_files(app: AppHandle, persist: PersistState) -> Result<Value, String> {
    let lang = lang_of(&persist);
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter(t(lang, "dialog.zsb", &[]), &["zsb"])
        .pick_files(move |p| {
            let _ = tx.send(p.map(|list| list.iter().map(|f| f.to_string()).collect::<Vec<_>>()));
        });
    let paths = match rx.recv() {
        Ok(Some(list)) if !list.is_empty() => list,
        _ => return Ok(json!({ "picked": false, "sealed": [], "errors": [] })),
    };
    let mut sealed: Vec<Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for path in &paths {
        let fname = std::path::Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        match fs::read_to_string(path)
            .map_err(|e| { let es = e.to_string(); t(lang, "err.import.read", &[("fname", &fname), ("e", &es)]) })
            .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|_| t(lang, "err.import.not_sealed", &[("fname", &fname)])))
        {
            Ok(doc) => {
                if doc.get("format").and_then(|v| v.as_str()) == Some(BUNDLE_FORMAT)
                    || doc.get("kdf").is_some()
                {
                    sealed.push(json!([fname, doc]));
                } else {
                    errors.push(t(lang, "err.import.not_sealed", &[("fname", &fname)]));
                }
            }
            Err(e) => errors.push(e),
        }
    }
    Ok(json!({ "picked": true, "sealed": sealed, "errors": errors }))
}

#[tauri::command]
pub fn import_sealed(
    app: AppHandle,
    persist: PersistState,
    files: Vec<Value>,
    password: String,
) -> Result<Value, String> {
    let _ = &app;
    let lang = lang_of(&persist);
    let mut added: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for entry in &files {
        let fname = entry
            .get(0)
            .and_then(|v| v.as_str())
            .unwrap_or("file")
            .to_string();
        let doc = entry.get(1).cloned().unwrap_or(Value::Null);
        match import_one(&persist, &doc, &password, &fname, lang) {
            Ok(name) => added.push(name),
            Err(e) => {
                if e.contains("already") || e.contains("已存在") {
                    skipped.push(fname);
                } else {
                    errors.push(e);
                }
            }
        }
    }
    persist.save_accounts(lang)?;
    Ok(json!({ "added": added, "skipped": skipped, "errors": errors }))
}

fn import_one(
    persist: &Arc<Persist>,
    doc: &Value,
    password: &str,
    fname: &str,
    lang: Lang,
) -> Result<String, String> {
    let sealed = doc.get("kdf").is_some();
    if !sealed {
        return Err(t(lang, "err.import.not_sealed_plain", &[]));
    }
    let plain = crate::crypto::open(doc, password).map_err(|e| {
        if e.contains("wrong password") {
            t(lang, "err.cipher.wrong_pw", &[])
        } else {
            t(lang, "err.cipher.bad_plain", &[("e", &e)])
        }
    })?;
    let bundle: Value = serde_json::from_slice(&plain)
        .map_err(|e| { let es = e.to_string(); t(lang, "err.cipher.bad_plain", &[("e", &es)]) })?;
    if bundle.get("format").and_then(|v| v.as_str()) != Some(BUNDLE_FORMAT) {
        return Err(t(lang, "err.import.not_bundle_plain", &[]));
    }
    let accounts = bundle
        .get("accounts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| t(lang, "err.bundle.no_accounts", &[]))?;
    let mut names = Vec::new();
    let mut g = persist.store.lock().unwrap();
    for entry in accounts {
        let creds = entry
            .get("credentials")
            .cloned()
            .filter(|v| v.is_object())
            .ok_or_else(|| t(lang, "err.bundle.no_creds", &[]))?;
        let name_in = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Imported")
            .to_string();
        // dedupe by hash
        let hash = entry
            .get("hash")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();
        if !hash.is_empty() && g.accounts.iter().any(|a| a.hash == hash) {
            return Err(t(lang, "err.import.dup", &[("fname", fname)]));
        }
        let mut base = name_in.clone();
        let mut n = 1;
        while g.accounts.iter().any(|a| a.name == base) {
            n += 1;
            base = format!("{name_in} {n}");
        }
        let now = crate::store::ts_now();
        let name = base.clone();
        g.accounts.push(Account {
            id: crate::oauth::new_id(),
            name: name.clone(),
            created_at: entry.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            updated_at: now,
            hash,
            credentials: creds,
            config: entry.get("config").cloned().filter(|v| v.is_object()),
            virtual_device_mid: entry
                .get("virtual_device_mid")
                .and_then(|v| v.as_str())
                .map(String::from),
            virtual_arms_uid: entry
                .get("virtual_arms_uid")
                .and_then(|v| v.as_str())
                .map(String::from),
        });
        names.push(name);
    }
    drop(g);
    match names.into_iter().next() {
        Some(n) => Ok(n),
        None => Err(t(lang, "err.import.no_creds", &[("fname", fname)])),
    }
}

// --------------------------------------------------------- background ops

pub mod auto {
    use super::*;

    /// Auto-switch: called periodically from lib.rs when the setting is on.
    /// Two consecutive "exhausted" readings switch to the best candidate.
    pub fn check_auto_switch(app: &AppHandle, persist: &Arc<Persist>) {
        let lang = persist.lang();
        let settings = persist.store.lock().unwrap().settings.clone();
        if !settings.auto_switch() {
            return;
        }
        let state = match get_state_no_emit(persist) {
            Ok(s) => s,
            Err(_) => return,
        };
        let active_id = state
            .get("active_account_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        let Some(active_id) = active_id else { return };
        let creds = {
            let g = persist.store.lock().unwrap();
            match g.accounts.iter().find(|a| a.id == active_id) {
                Some(a) => a.credentials.clone(),
                None => return,
            }
        };
        let quota = match crate::quota::query(&creds, lang) {
            Ok(q) => q,
            Err(_) => return, // network errors don't count as exhaustion
        };
        let exhausted = quota
            .items
            .iter()
            .filter(|i| i.kind != "grant")
            .filter_map(|i| i.remaining)
            .all(|r| r <= 0.0)
            && !quota.items.iter().all(|i| i.remaining.is_none());
        if !exhausted {
            *exhaust_streak().lock().unwrap() = 0;
            return;
        }
        let mut streak = exhaust_streak().lock().unwrap();
        *streak += 1;
        if *streak < 2 {
            return;
        }
        *streak = 0;
        // candidate: another account with confirmed remaining quota + config snapshot
        let candidates: Vec<Account> = persist.store.lock().unwrap().accounts.clone();
        let candidate = candidates
            .iter()
            .filter(|a| a.id != active_id && a.config.is_some())
            .find_map(|a| {
                let q = crate::quota::query(&a.credentials, lang).ok()?;
                let has = q
                    .items
                    .iter()
                    .filter(|i| i.kind != "grant")
                    .filter_map(|i| i.remaining)
                    .any(|r| r > 0.0);
                if has {
                    Some(a.clone())
                } else {
                    None
                }
            });
        let Some(candidate) = candidate else {
            let _ = app.emit("auto-switch-result", json!({"status": "no-candidate"}));
            return;
        };
        match switch_to_core(persist, candidate.id.clone(), true, true) {
            Ok(res) => {
                let name = candidate.name.clone();
                let launch_error = res
                    .get("launch_error")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let config_stale = res
                    .get("config_stale")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let _ = app.emit(
                    "auto-switch-result",
                    json!({
                        "status": "switched", "name": name,
                        "launchError": launch_error, "configStale": config_stale,
                    }),
                );
            }
            Err(e) => {
                let _ = app.emit(
                    "auto-switch-result",
                    json!({"status": "error", "error": e.to_string()}),
                );
            }
        }
    }

    fn get_state_no_emit(persist: &Arc<Persist>) -> Result<Value, String> {
        // reuse get_state logic without needing an AppHandle
        let lang = persist.lang();
        let g = persist.store.lock().unwrap();
        let live_hash = if zcode::credentials_exists() {
            zcode::read_json_file(&zcode::credentials_path(), lang)
                .map(|v| zcode::hash_json(&v))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let active_id = g
            .accounts
            .iter()
            .find(|a| !live_hash.is_empty() && a.hash == live_hash)
            .map(|a| a.id.clone());
        let _ = lang;
        Ok(json!({ "active_account_id": active_id }))
    }

    fn exhaust_streak() -> &'static std::sync::Mutex<u32> {
        use std::sync::OnceLock;
        static STREAK: OnceLock<std::sync::Mutex<u32>> = OnceLock::new();
        STREAK.get_or_init(|| std::sync::Mutex::new(0))
    }
}

// ---------------------------------------------------------------- oauth

#[tauri::command]
pub fn oauth_providers(persist: PersistState) -> Value {
    let lang = lang_of(&persist);
    Value::Array(vec![
        json!({
            "id": "bigmodel",
            "display": t(lang, "prov.bigmodel", &[]),
        }),
        json!({
            "id": "zai",
            "display": t(lang, "prov.zai", &[]),
        }),
    ])
}

#[tauri::command]
pub fn oauth_begin(app: AppHandle, persist: PersistState, provider: String) -> Result<(), String> {
    crate::oauth::begin(app, persist.inner().clone(), &provider)
}
