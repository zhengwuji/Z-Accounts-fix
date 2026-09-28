//! ZCode desktop integration: data-dir discovery, credentials/config读写,
//! device-identity virtualization, process management, hot/restart switching,
//! and recent model-request status parsing.

use crate::i18n::{t, Lang};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const ZCODE_EXE: &str = "ZCode.exe";
const KILL_TIMEOUT: Duration = Duration::from_secs(10);
const HOT_VERIFY_RETRIES: usize = 3;

// ---------------------------------------------------------------- paths

pub fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// ZCode's data dir (`~/.zcode/v2`), honoring `ZCODE_DATA_BASE_DIR`.
pub fn zcode_data_dir() -> PathBuf {
    if let Ok(base) = std::env::var("ZCODE_DATA_BASE_DIR") {
        if !base.trim().is_empty() {
            return PathBuf::from(base);
        }
    }
    home_dir().join(".zcode").join("v2")
}

pub fn credentials_path() -> PathBuf {
    zcode_data_dir().join("credentials.json")
}

pub fn config_path() -> PathBuf {
    zcode_data_dir().join("config.json")
}

pub fn setting_path() -> PathBuf {
    zcode_data_dir().join("setting.json")
}

pub fn telemetry_path() -> PathBuf {
    zcode_data_dir().join("telemetry-state.json")
}

/// `%APPDATA%\ZCode` — ZCode desktop's Electron userData dir.
pub fn zcode_appdata_dir() -> PathBuf {
    std::env::var("APPDATA")
        .map(|a| PathBuf::from(a).join("ZCode"))
        .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming").join("ZCode"))
}

/// Default install location of the ZCode desktop executable.
pub fn default_exe_path() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join("AppData").join("Local"));
    local.join("Programs").join("ZCode").join(ZCODE_EXE)
}

/// Locate ZCode.exe: explicit path → default install dir → registry uninstall
/// entries under HKCU/HKLM.
pub fn find_zcode_exe(configured: &str) -> Option<PathBuf> {
    if !configured.trim().is_empty() {
        let p = PathBuf::from(configured);
        if p.is_file() {
            return Some(p);
        }
    }
    let default = default_exe_path();
    if default.is_file() {
        return Some(default);
    }
    registry_uninstall_lookup()
}

fn registry_uninstall_lookup() -> Option<PathBuf> {
    const KEYS: &[&str] = &[
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];
    const HIVES: &[(&str, &str)] = &[("HKCU", ""), ("HKLM", "")];
    for (hive, _) in HIVES {
        for key in KEYS {
            let full = format!(r"{hive}\{key}");
            let mut reg_cmd = Command::new("reg");
            if let Ok(out) = silent(&mut reg_cmd)
                .args(["query", &full, "/s", "/f", "ZCode.exe", "/d"])
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let line = line.trim();
                    if line.ends_with(ZCODE_EXE) && line.contains("REG_SZ") {
                        if let Some(path) = line.split_whitespace().last() {
                            let p = PathBuf::from(path);
                            if p.is_file() {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------- process

/// CREATE_NO_WINDOW — console helpers spawned from a GUI app must not flash
/// a console window (tray thread polls every few seconds).
#[cfg(windows)]
pub fn silent(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000)
}
#[cfg(not(windows))]
pub fn silent(cmd: &mut Command) -> &mut Command {
    cmd
}

pub fn is_running() -> bool {
    let mut cmd = Command::new("tasklist");
    let out = silent(&mut cmd)
        .args(["/FI", &format!("IMAGENAME eq {ZCODE_EXE}"), "/FO", "CSV", "/NH"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains(ZCODE_EXE),
        Err(_) => false,
    }
}

/// Kill ZCode.exe, wait until it is really gone. Ok(true) = was running & killed,
/// Ok(false) = was not running.
pub fn kill(lang: Lang) -> Result<bool, String> {
    if !is_running() {
        return Ok(false);
    }
    let mut cmd = Command::new("taskkill");
    let _ = silent(&mut cmd)
        .args(["/F", "/IM", ZCODE_EXE, "/T"])
        .output();
    let start = Instant::now();
    while start.elapsed() < KILL_TIMEOUT {
        if !is_running() {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(t(lang, "err.zcode.kill_timeout", &[]))
}

pub fn launch(exe: &Path) -> Result<(), String> {
    let mut cmd = Command::new(exe);
    silent(&mut cmd)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- live files

pub fn read_json_file(path: &Path, lang: Lang) -> Result<Value, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| { let es = e.to_string(); t(lang, "err.read", &[("path", &path.display().to_string()), ("e", &es)]) })?;
    serde_json::from_str(&raw)
        .map_err(|e| { let es = e.to_string(); t(lang, "err.bad_json", &[("path", &path.display().to_string()), ("e", &es)]) })
}

pub fn credentials_exists() -> bool {
    credentials_path().is_file()
}

/// Does the live credentials.json carry real login state?
pub fn live_has_credentials(creds: &Value) -> bool {
    if !creds.is_object() || creds.as_object().unwrap().is_empty() {
        return false;
    }
    // any token-ish key with a non-empty string value
    creds.as_object().unwrap().values().any(|v| {
        v.as_str().map(|s| !s.trim().is_empty()).unwrap_or(false)
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use ring::digest;
    let digest = digest::digest(&digest::SHA256, bytes);
    hex(digest.as_ref())
}

pub fn hash_file(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|b| sha256_hex(&b))
}

/// Canonical live hash: SHA-256 of the compact-serialized parsed credentials
/// (whitespace-insensitive, matches the original app's semantics).
pub fn hash_json(v: &serde_json::Value) -> String {
    match serde_json::to_string(v) {
        Ok(s) => sha256_hex(s.as_bytes()),
        Err(_) => String::new(),
    }
}

/// Read the live credentials.json and return (parsed, hash).
pub fn live_credentials(lang: Lang) -> Result<(serde_json::Value, String), String> {
    let path = credentials_path();
    let raw = fs::read_to_string(&path)
        .map_err(|e| t(lang, "err.read", &[("path", &path.display().to_string()), ("e", &e.to_string())]))?;
    let v: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| t(lang, "err.bad_json", &[("path", &path.display().to_string()), ("e", &e.to_string())]))?;
    let hash = hash_json(&v);
    Ok((v, hash))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ------------------------------------------------------- device identity

fn rand_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn rand_arms_uid() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 16];
    let _ = rng.fill(&mut bytes);
    let suffix: String = bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    format!("uid_{suffix}")
}

/// Current deviceMid from telemetry-state.json (if present).
pub fn current_device_mid() -> Option<String> {
    read_json_file(&telemetry_path(), Lang::Zh)
        .ok()
        .and_then(|v| v.get("deviceMid").and_then(|m| m.as_str().map(String::from)))
}

/// Current `_arms_uid` from ZCode desktop's rum-electron-store.
pub fn current_arms_uid() -> Option<String> {
    for entry in rum_store_files() {
        if let Ok(v) = read_json_file(&entry, Lang::Zh) {
            if let Some(uid) = v.get("_arms_uid").and_then(|x| x.as_str()) {
                return Some(uid.to_string());
            }
        }
    }
    None
}

fn rum_store_files() -> Vec<PathBuf> {
    let dir = zcode_appdata_dir().join("rum-electron-store");
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "json").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Write deviceMid into telemetry-state.json (preserving other fields).
pub fn write_device_mid(mid: &str, lang: Lang) -> Result<(), String> {
    let path = telemetry_path();
    let mut doc = if path.is_file() {
        read_json_file(&path, lang).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };
    if !doc.is_object() {
        doc = json!({});
    }
    doc["deviceMid"] = json!(mid);
    let pretty = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| { let es = e.to_string(); t(lang, "err.write_file", &[("path", &path.display().to_string()), ("e", &es)]) })
}

/// Write `_arms_uid` into ZCode desktop's rum-electron-store file(s).
pub fn write_arms_uid(uid: &str, lang: Lang) -> Result<(), String> {
    let files = rum_store_files();
    if files.is_empty() {
        let dir = zcode_appdata_dir().join("rum-electron-store");
        let _ = fs::create_dir_all(&dir);
        // electron-store names the file base64("default")
        let name = crate::b64url_nopad("default");
        files.iter().for_each(|_| {});
        let path = dir.join(format!("{name}.json"));
        let doc = json!({ "_arms_uid": uid });
        let pretty = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
        return fs::write(&path, pretty)
            .map_err(|e| { let es = e.to_string(); t(lang, "err.write_file", &[("path", &path.display().to_string()), ("e", &es)]) });
    }
    let mut last_err = None;
    for path in &files {
        let mut doc = read_json_file(path, lang).unwrap_or_else(|_| json!({}));
        if !doc.is_object() {
            continue;
        }
        if doc.get("_arms_uid").is_none() {
            continue; // only touch stores that actually carry an arms uid
        }
        doc["_arms_uid"] = json!(uid);
        if let Ok(pretty) = serde_json::to_string_pretty(&doc) {
            if let Err(e) = fs::write(path, pretty) {
                last_err = Some({ let es = e.to_string(); t(lang, "err.write_file", &[("path", &path.display().to_string()), ("e", &es)]) });
            }
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Mint-or-take the device identity for an account and apply it to ZCode.
pub fn apply_device_identity(
    mid: &mut Option<String>,
    arms: &mut Option<String>,
    lang: Lang,
) -> Result<(), String> {
    if mid.is_none() {
        *mid = current_device_mid().or_else(|| Some(rand_uuid()));
    }
    if arms.is_none() {
        *arms = current_arms_uid().or_else(|| Some(rand_arms_uid()));
    }
    write_device_mid(mid.as_deref().unwrap_or_default(), lang)?;
    write_arms_uid(arms.as_deref().unwrap_or_default(), lang)?;
    Ok(())
}

/// Align ZCode's `setting.json` → `providerFamilyDomain` with the account's
/// active provider (best effort; parse failures are ignored on purpose).
pub fn align_family_domain(creds: &Value) {
    let provider = creds
        .get("oauth:active_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if provider.is_empty() {
        return;
    }
    let path = setting_path();
    let Ok(raw) = fs::read_to_string(&path) else { return };
    let Ok(mut doc) = serde_json::from_str::<Value>(&raw) else {
        // setting.json 解析失败, 跳过 family domain 对齐
        return;
    };
    if doc.get("providerFamilyDomain").and_then(|v| v.as_str()) == Some(provider) {
        return;
    }
    doc["providerFamilyDomain"] = json!(provider);
    doc["providerFamilyDomainUpdatedAt"] = json!(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    );
    if let Ok(pretty) = serde_json::to_string_pretty(&doc) {
        let _ = fs::write(&path, pretty);
    }
}

// ---------------------------------------------------------------- switching

pub struct SwitchOutcome {
    pub already_active: bool,
    pub hot: bool,
    pub killed: bool,
    pub preserved_as: Option<String>,
    pub launched: bool,
    pub launch_error: Option<String>,
    pub config_stale: bool,
}

/// Preserve the current live login into the store (auto-backup before switch).
pub fn preserve_live_name() -> String {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    format!("Backup-{ts}")
}

/// Hot-swap: write credentials while ZCode is running, verifying the file was
/// not concurrently rewritten (3 retries), else fail with err.hot.verify.
pub fn hot_write(path: &Path, content: &[u8], lang: Lang) -> Result<(), String> {
    for _ in 0..HOT_VERIFY_RETRIES {
        let before = hash_file(path).unwrap_or_default();
        fs::write(path, content)
            .map_err(|e| { let es = e.to_string(); t(lang, "err.write_config", &[("e", &es)]) })?;
        thread::sleep(Duration::from_millis(150));
        let after = hash_file(path).unwrap_or_default();
        if after == sha256_hex(content) || after == before && content.is_empty() {
            // stable — either our content stuck or ZCode rewrote identical state
            return Ok(());
        }
        if before.is_empty() {
            return Ok(());
        }
    }
    Err(t(lang, "err.hot.verify", &[]))
}

// ------------------------------------------------------- model status log

#[derive(Clone, Debug, Default)]
pub struct ModelStatus {
    pub kind: String, // gateway_blocked | rate_limited
    pub at: i64,      // unix ms
    pub model: String,
    pub provider: String,
    pub status_code: Option<i64>,
    pub provider_code: Option<String>,
    pub request_id: String,
}

fn parse_log_ts(v: &Value) -> i64 {
    v.get("timestamp")
        .and_then(|x| x.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp_millis())
        .unwrap_or(0)
}

fn classify(v: &Value) -> Option<ModelStatus> {
    let event = v.get("event").and_then(|x| x.as_str())?;
    if event != "model.request.completed" && event != "turn.failed" {
        return None;
    }
    let ctx = v.get("context").cloned().unwrap_or_else(|| json!({}));
    let message = v
        .get("message")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_lowercase();
    let resp_status = ctx
        .get("responseStatus")
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())));
    let provider_code = ctx
        .get("providerCode")
        .and_then(|x| x.as_str())
        .map(String::from);
    let joined = format!(
        "{} {} {} {}",
        message,
        provider_code.clone().unwrap_or_default(),
        ctx.get("error").and_then(|x| x.as_str()).unwrap_or(""),
        ctx.get("cause").and_then(|x| x.as_str()).unwrap_or("")
    )
    .to_lowercase();
    let kind = if joined.contains("gateway_blocked") || joined.contains("unusual activity") {
        "gateway_blocked"
    } else if joined.contains("rate_limited") || joined.contains("rate limit") || resp_status == Some(429) {
        "rate_limited"
    } else {
        return None;
    };
    Some(ModelStatus {
        kind: kind.into(),
        at: parse_log_ts(v),
        model: ctx
            .get("modelId")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .into(),
        provider: ctx
            .get("providerId")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .into(),
        status_code: resp_status,
        provider_code,
        request_id: ctx
            .get("requestId")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .into(),
    })
}

/// Scan the newest ZCode CLI logs for the most recent throttled/blocked model
/// request (`~/.zcode/cli/log/zcode-*.jsonl`).
pub fn recent_model_status() -> Option<ModelStatus> {
    let log_dir = home_dir().join(".zcode").join("cli").join("log");
    let mut files: Vec<PathBuf> = fs::read_dir(&log_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "jsonl").unwrap_or(false))
        .collect();
    files.sort();
    files.reverse();
    let mut best: Option<ModelStatus> = None;
    for file in files.iter().take(2) {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines().rev() {
                let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
                if let Some(st) = classify(&v) {
                    if best.as_ref().map(|b| st.at > b.at).unwrap_or(true) {
                        best = Some(st);
                    }
                    break; // newest per file
                }
            }
        }
        if best.is_some() {
            break;
        }
    }
    best
}
