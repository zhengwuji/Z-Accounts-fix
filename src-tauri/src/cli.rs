//! Headless CLI mode: `Z-Accounts.exe <subcommand> …` runs without the GUI.
//!
//! Subcommands: state | list | capture | rename | delete | update | switch |
//! quota | claim-preview | kill | export | export-all | import | behavior |
//! setpath | launch

use crate::i18n::{t, Lang};
use crate::store::{Account, Persist, BUNDLE_FORMAT};
use crate::zcode;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn run(args: Vec<String>) -> i32 {
    let persist = match Persist::load() {
        Ok(p) => std::sync::Arc::new(p),
        Err(e) => {
            println!("{{\"error\":\"{}\"}}", e);
            return 1;
        }
    };
    let lang = persist.lang();
    let Some(cmd) = args.first().cloned() else {
        eprintln!("{}", t(lang, "cli.missing_cmd", &[]));
        return 2;
    };
    let flags = parse_flags(&args[1..]);
    let result: Result<Value, String> = match cmd.as_str() {
        "state" => crate::commands::get_state_core(&persist),
        "list" => {
            let g = persist.store.lock().unwrap();
            Ok(json!(g
                .accounts
                .iter()
                .map(|a| json!({
                    "id": a.id, "name": a.name, "created_at": a.created_at,
                    "hash": a.hash,
                }))
                .collect::<Vec<_>>()))
        }
        "capture" => crate::commands::capture_current_core(
            &persist,
            flags.get("name").map(|s| s.to_string()),
        ),
        "rename" => match (flags.get("id"), flags.get("name")) {
            (Some(id), Some(name)) => {
                crate::commands::rename_account_core(&persist, id.clone(), name.clone())
            }
            _ => return usage(lang, "cli.usage.rename"),
        },
        "delete" => match flags.get("id") {
            Some(id) => crate::commands::delete_account_core(&persist, id.clone())
                .map(|_| json!({"ok": true})),
            _ => return usage(lang, "cli.usage.delete"),
        },
        "update" => match flags.get("id") {
            Some(id) => crate::commands::update_account_from_live_core(&persist, id.clone()),
            _ => return usage(lang, "cli.usage.update"),
        },
        "switch" => match flags.get("id") {
            Some(id) => crate::commands::switch_to_core(
                &persist,
                id.clone(),
                flags.contains_key("force"),
                !flags.contains_key("no-restart"),
            ),
            _ => return usage(lang, "cli.usage.switch"),
        },
        "quota" => match flags.get("id") {
            Some(id) => crate::commands::get_account_quota_core(&persist, id.clone()),
            None => match live_quota(&persist) {
                Some(q) => Ok(q),
                None => Err(t(lang, "err.quota.no_token", &[])),
            },
        },
        "claim-preview" => match flags.get("id") {
            Some(id) => crate::commands::claim_preview_core(&persist, id.clone()),
            _ => return usage(lang, "cli.usage.switch"),
        },
        "kill" => crate::commands::kill_zcode_core(&persist).map(|_| json!({"ok": true})),
        "export" => {
            let Some(out) = flags.get("out") else { return usage(lang, "cli.usage.export") };
            let Some(id) = flags.get("id") else { return usage(lang, "cli.usage.export") };
            let Some(password) = password(&flags) else {
                eprintln!("{}", t(lang, "cli.pw_hint", &[]));
                return 2;
            };
            crate::commands::export_finalize_core(&persist, out.clone(), id.clone(), password)
        }
        "export-all" => {
            let Some(out) = flags.get("out") else { return usage(lang, "cli.usage.export_all") };
            let Some(password) = password(&flags) else {
                eprintln!("{}", t(lang, "cli.pw_hint", &[]));
                return 2;
            };
            crate::commands::export_all_finalize_core(&persist, out.clone(), password)
        }
        "import" => {
            let Some(file) = flags.get("file") else { return usage(lang, "cli.usage.import") };
            let Some(password) = password(&flags) else {
                eprintln!("{}", t(lang, "cli.pw_hint", &[]));
                return 2;
            };
            import_file(&persist, file, &password, lang)
        }
        "behavior" => {
            crate::commands::set_behavior_core(
                &persist,
                bool_flag(&flags, "launch"),
                bool_flag(&flags, "tray"),
                bool_flag(&flags, "hot"),
                bool_flag(&flags, "autoclaim"),
                bool_flag(&flags, "autoswitch"),
            )
            .map(|_| json!({"ok": true}))
        }
        "setpath" => match flags.get("path") {
            Some(path) => crate::commands::set_zcode_path_core(&persist, path.clone())
                .map(|_| json!({"ok": true})),
            _ => return usage(lang, "cli.usage.setpath"),
        },
        "launch" => crate::commands::launch_zcode_core(&persist).map(|_| json!({"ok": true})),
        other => {
            eprintln!("{}", t(lang, "cli.unknown_cmd", &[("cmd", other)]));
            return 2;
        }
    };
    match result {
        Ok(v) => {
            print_json(v);
            0
        }
        Err(e) => {
            println!("{{\"error\":{}}}", serde_json::to_string(&e).unwrap_or_default());
            1
        }
    }
}

fn bool_flag(flags: &HashMap<String, String>, name: &str) -> Option<bool> {
    flags
        .get(name)
        .map(|s| s == "true" || s == "1" || s == "on" || s.is_empty())
}

fn usage(lang: Lang, key: &str) -> i32 {
    eprintln!("{}", t(lang, key, &[]));
    2
}

fn password(flags: &HashMap<String, String>) -> Option<String> {
    std::env::var("ZSW_PASSWORD")
        .ok()
        .or_else(|| flags.get("password").cloned())
}

fn parse_flags(args: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(name) = a.clone().strip_prefix("--").map(String::from) {
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                map.insert(name, args[i + 1].clone());
                i += 2;
            } else {
                map.insert(name, String::new());
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    map
}

fn print_json(v: Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
}

fn live_quota(persist: &std::sync::Arc<Persist>) -> Option<Value> {
    let lang = persist.lang();
    let creds = zcode::read_json_file(&zcode::credentials_path(), lang).ok()?;
    crate::quota::query(&creds, lang)
        .ok()
        .map(|q| q.to_json(chrono::Utc::now().timestamp_millis()))
}

fn import_file(
    persist: &std::sync::Arc<Persist>,
    file: &str,
    password: &str,
    lang: Lang,
) -> Result<Value, String> {
    let raw = std::fs::read_to_string(file)
        .map_err(|e| { let es = e.to_string(); t(lang, "cli.read_fail", &[("e", &es)]) })?;
    let doc: Value = serde_json::from_str(&raw)
        .map_err(|e| { let es = e.to_string(); t(lang, "cli.json_fail", &[("e", &es)]) })?;
    if doc.get("format").and_then(|v| v.as_str()) != Some(BUNDLE_FORMAT) {
        return Err(t(lang, "err.import.not_sealed_plain", &[]));
    }
    let plain = crate::crypto::open(&doc, password).map_err(|e| {
        if e.contains("wrong password") {
            t(lang, "err.cipher.wrong_pw", &[])
        } else {
            e
        }
    })?;
    let bundle: Value = serde_json::from_slice(&plain)
        .map_err(|e| { let es = e.to_string(); t(lang, "cli.json_fail", &[("e", &es)]) })?;
    let accounts = bundle
        .get("accounts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| t(lang, "err.bundle.no_accounts", &[]))?;
    let mut added = 0;
    let mut skipped = 0;
    {
        let mut g = persist.store.lock().unwrap();
        for entry in accounts {
            let hash = entry.get("hash").and_then(|v| v.as_str()).unwrap_or("");
            if !hash.is_empty() && g.accounts.iter().any(|a| a.hash == hash) {
                skipped += 1;
                continue;
            }
            let creds = entry
                .get("credentials")
                .cloned()
                .ok_or_else(|| t(lang, "err.bundle.no_creds", &[]))?;
            let mut base = entry
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Imported")
                .to_string();
            let mut n = 1;
            while g.accounts.iter().any(|a| a.name == base) {
                n += 1;
                base = format!("{base} {n}");
            }
            let now = crate::store::ts_now();
            g.accounts.push(Account {
                id: crate::oauth::new_id(),
                name: base,
                created_at: entry
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: now,
                hash: hash.to_string(),
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
            added += 1;
        }
    }
    persist.save_accounts(lang)?;
    Ok(json!({"added": added, "skipped": skipped}))
}
