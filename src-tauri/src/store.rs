//! Account store + settings, byte-layout compatible with the original app:
//!
//!   <data_dir>/accounts/<uuid>.json   — one file per account
//!   <data_dir>/settings.json          — tool settings (nullable fields)
//!   oauth.log → %LOCALAPPDATA%\com.zaccounts.app\logs\oauth.log
//!
//! `<data_dir>` = `ZCODE_SWITCH_HOME` override, else `~/.zcode-switch`.

use crate::i18n::{t, Lang};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const BUNDLE_FORMAT: &str = "zsw-accounts-bundle";

pub fn ts_now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// Normalize an account document: older versions also wrote camelCase
/// `createdAt`/`updatedAt` duplicates, which serde would reject as duplicate
/// fields — fold them into the snake_case keys first.
pub fn to_account(mut v: Value) -> Option<Account> {
    if let Some(obj) = v.as_object_mut() {
        if !obj.contains_key("created_at") {
            if let Some(c) = obj.remove("createdAt") {
                obj.insert("created_at".into(), c);
            }
        } else {
            obj.remove("createdAt");
        }
        if !obj.contains_key("updated_at") {
            if let Some(c) = obj.remove("updatedAt") {
                obj.insert("updated_at".into(), c);
            }
        } else {
            obj.remove("updatedAt");
        }
    }
    serde_json::from_value::<Account>(v).ok()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    /// local timestamp strings, e.g. "2026-09-26 00:45"
    pub created_at: String,
    pub updated_at: String,
    /// SHA-256 (hex) of the live credentials.json bytes this account was saved from
    pub hash: String,
    /// Contents of ZCode's credentials.json
    pub credentials: Value,
    /// Contents of ZCode's config.json (None → no snapshot)
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default)]
    pub virtual_device_mid: Option<String>,
    #[serde(default)]
    pub virtual_arms_uid: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    #[serde(default)]
    pub zcode_path: Option<String>,
    #[serde(default)]
    pub launch_after_switch: Option<bool>,
    #[serde(default)]
    pub close_to_tray: Option<bool>,
    #[serde(default)]
    pub hot_switch: Option<bool>,
    #[serde(default)]
    pub auth_proxy_on: Option<bool>,
    #[serde(default)]
    pub auth_proxy_url: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub auto_claim: Option<bool>,
    #[serde(default)]
    pub auto_switch: Option<bool>,
}

impl Settings {
    pub fn lang(&self) -> Lang {
        Lang::from_str(self.language.as_deref().unwrap_or("zh"))
    }
    pub fn theme(&self) -> String {
        match self.theme.as_deref() {
            Some("light") => "light".into(),
            _ => "dark".into(),
        }
    }
    pub fn launch_after_switch(&self) -> bool {
        self.launch_after_switch.unwrap_or(true)
    }
    pub fn close_to_tray(&self) -> bool {
        self.close_to_tray.unwrap_or(true)
    }
    pub fn hot_switch(&self) -> bool {
        self.hot_switch.unwrap_or(false)
    }
    pub fn zcode_path(&self) -> &str {
        self.zcode_path.as_deref().unwrap_or("")
    }
    pub fn proxy_on(&self) -> bool {
        self.auth_proxy_on.unwrap_or(false)
    }
    pub fn proxy_url(&self) -> &str {
        self.auth_proxy_url.as_deref().unwrap_or("")
    }
    pub fn auto_claim(&self) -> bool {
        self.auto_claim.unwrap_or(false)
    }
    pub fn auto_switch(&self) -> bool {
        self.auto_switch.unwrap_or(false)
    }
}

#[derive(Default)]
pub struct Store {
    pub accounts: Vec<Account>,
    pub settings: Settings,
}

pub struct Persist {
    pub dir: PathBuf,
    pub log_dir: PathBuf,
    pub store: Mutex<Store>,
}

impl Persist {
    /// Resolve the data dir: `ZCODE_SWITCH_HOME` overrides, else `~/.zcode-switch`.
    pub fn data_dir() -> Result<PathBuf, String> {
        if let Ok(home) = std::env::var("ZCODE_SWITCH_HOME") {
            if !home.trim().is_empty() {
                return Ok(PathBuf::from(home));
            }
        }
        Ok(crate::zcode::home_dir().join(".zcode-switch"))
    }

    /// Backend log dir: `%LOCALAPPDATA%\com.zaccounts.app\logs` (as the original).
    fn log_dir() -> PathBuf {
        std::env::var("LOCALAPPDATA")
            .map(|l| PathBuf::from(l).join("com.zaccounts.app").join("logs"))
            .unwrap_or_else(|_| Persist::data_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn lang(&self) -> Lang {
        self.store.lock().unwrap().settings.lang()
    }

    pub fn load() -> Result<Persist, String> {
        let dir = Self::data_dir()?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let accounts_dir = dir.join("accounts");
        fs::create_dir_all(&accounts_dir).map_err(|e| e.to_string())?;
        let log_dir = Self::log_dir();
        let _ = fs::create_dir_all(&log_dir);

        let settings = load_json(&dir.join("settings.json"))
            .and_then(|v| serde_json::from_value::<Settings>(v).map_err(|e| e.to_string()))
            .unwrap_or_default();

        let mut accounts = Vec::new();
        if let Ok(rd) = fs::read_dir(&accounts_dir) {
            let mut paths: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
                .collect();
            paths.sort();
            for p in paths {
                if let Ok(v) = load_json(&p) {
                    match to_account(v) {
                        Some(a) => accounts.push(a),
                        None => {
                            // corrupt archive — skip the file, keep the rest
                        }
                    }
                }
            }
        }
        accounts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(Persist {
            dir,
            log_dir,
            store: Mutex::new(Store {
                accounts,
                settings,
            }),
        })
    }

    pub fn save_accounts(&self, lang: Lang) -> Result<(), String> {
        let g = self.store.lock().unwrap();
        let accounts_dir = self.dir.join("accounts");
        fs::create_dir_all(&accounts_dir)
            .map_err(|e| t(lang, "err.mkdir", &[("e", &e.to_string())]))?;
        let mut keep: Vec<String> = Vec::new();
        for a in &g.accounts {
            keep.push(format!("{}.json", a.id));
            let v =
                serde_json::to_value(a).map_err(|e| t(lang, "err.serialize", &[("e", &e.to_string())]))?;
            let path = accounts_dir.join(format!("{}.json", a.id));
            save_json(&path, &v, lang)?;
        }
        // remove files that no longer correspond to an account
        if let Ok(rd) = fs::read_dir(&accounts_dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".json") && !keep.contains(&name) {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
        Ok(())
    }

    pub fn save_settings(&self, lang: Lang) -> Result<(), String> {
        let g = self.store.lock().unwrap();
        let v = serde_json::to_value(&g.settings).unwrap_or(Value::Null);
        save_json(&self.dir.join("settings.json"), &v, lang)
    }

    /// Append a line to oauth.log (rotating the previous log to oauth.log.old).
    pub fn oauth_log(&self, line: &str) {
        let path = self.log_dir.join("oauth.log");
        if let Ok(m) = fs::metadata(&path) {
            if m.len() > 512 * 1024 {
                let _ = fs::rename(&path, self.log_dir.join("oauth.log.old"));
            }
        }
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "[{ts}] {line}");
        }
    }
}

fn load_json(path: &Path) -> Result<Value, String> {
    let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

pub fn save_json(path: &Path, v: &Value, lang: Lang) -> Result<(), String> {
    let pretty = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| {
            let es = e.to_string();
            t(lang, "err.write_file", &[("path", &path.display().to_string()), ("e", &es)])
        })?;
        f.write_all(pretty.as_bytes()).map_err(|e| {
            let es = e.to_string();
            t(lang, "err.write_file", &[("path", &path.display().to_string()), ("e", &es)])
        })?;
    }
    fs::rename(&tmp, path).map_err(|e| {
        let es = e.to_string();
        t(lang, "err.rename_fail", &[("path", &path.display().to_string()), ("e", &es)])
    })
}

/// Validate an account name (uniqueness checked by callers).
pub fn validate_name(lang: Lang, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(t(lang, "err.name.empty", &[]));
    }
    if name.chars().count() > 40 {
        return Err(t(lang, "err.name.too_long", &[]));
    }
    Ok(())
}

pub fn name_taken(lang: Lang, name: &str, other: &str) -> String {
    t(lang, "err.name.taken", &[("name", name), ("other", other)])
}
