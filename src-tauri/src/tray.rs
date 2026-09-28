//! Tray icon + menu. Rebuilt whenever the app state changes so the tooltip and
//! menu items reflect the live status (未保存的登录 / 未登录 / 可安全切换).

use crate::i18n::t;
use crate::store::Persist;
use serde_json::json;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

// rebuild() is called from setup, the 5s status-poll thread and menu handlers
// (main thread). Tauri keeps tray icons in a plain Vec, so two concurrent
// calls would both miss tray_by_id() and build two icons that never get
// deduplicated. Serialize the check-then-create sequence; try_lock so a
// caller never blocks while the lock holder is queued on the main thread
// (the poll thread builds via run_on_main_thread) — skipping is safe because
// the next poll refreshes the tooltip anyway.
static TRAY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn rebuild(app: &AppHandle, persist: &Arc<Persist>) -> tauri::Result<()> {
    let Ok(_guard) = TRAY_LOCK.get_or_init(|| Mutex::new(())).try_lock() else {
        return Ok(());
    };
    let lang = persist.lang();
    let running = crate::zcode::is_running();
    let live_logged = crate::zcode::credentials_exists()
        && crate::zcode::read_json_file(&crate::zcode::credentials_path(), lang)
            .map(|v| crate::zcode::live_has_credentials(&v))
            .unwrap_or(false);
    let live_hash = if crate::zcode::credentials_exists() {
        crate::zcode::read_json_file(&crate::zcode::credentials_path(), lang)
            .map(|v| crate::zcode::hash_json(&v))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let known = persist
        .store
        .lock()
        .unwrap()
        .accounts
        .iter()
        .any(|a| a.hash == live_hash && !live_hash.is_empty());
    let status = if running {
        t(lang, "m.status.running", &[])
    } else if live_logged && !known {
        t(lang, "tray.unsaved", &[])
    } else if !live_logged {
        t(lang, "tray.logged_out", &[])
    } else {
        t(lang, "m.status.safe", &[])
    };

    let show = MenuItem::with_id(app, "show", t(lang, "tray.show", &[]), true, None::<&str>)?;
    let capture = MenuItem::with_id(app, "capture", t(lang, "tray.capture", &[]), live_logged && !known, None::<&str>)?;
    let launch = MenuItem::with_id(app, "launch", t(lang, "tray.launch", &[]), !running, None::<&str>)?;
    let kill = MenuItem::with_id(app, "kill", t(lang, "tray.kill", &[]), running, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", t(lang, "tray.quit", &[]), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &capture, &launch, &kill, &quit])?;

    let tooltip = format!("Z-Accounts · {status}");
    if let Some(existing) = app.tray_by_id("main-tray") {
        existing.set_menu(Some(menu))?;
        existing.set_tooltip(Some(tooltip))?;
        return Ok(());
    }
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(tooltip)
        .on_menu_event(|app, event| {
            handle_menu(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    let persist = app.state::<Arc<Persist>>();
    let lang = persist.lang();
    match id {
        "show" => show_main(app),
        "quit" => {
            app.exit(0);
        }
        "launch" => {
            let exe = {
                let g = persist.store.lock().unwrap();
                crate::zcode::find_zcode_exe(g.settings.zcode_path())
            };
            if let Some(exe) = exe {
                let _ = crate::zcode::launch(&exe);
            }
            let app2 = app.clone();
            let persist2 = persist.inner().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1200));
                rebuild(&app2, &persist2).ok();
            });
        }
        "kill" => {
            let _ = crate::zcode::kill(lang);
            rebuild(app, &persist).ok();
        }
        "capture" => {
            let result = crate::commands::capture_current_core(persist.inner(), None);
            let payload = match &result {
                Ok(v) => json!({"action": "capture", "ok": true, "result": v, "error": serde_json::Value::Null}),
                Err(e) => json!({"action": "capture", "ok": false, "result": serde_json::Value::Null, "error": e}),
            };
            let _ = app.emit("tray-action", payload);
            let _ = result;
            rebuild(app, &persist).ok();
        }
        _ => {}
    }
}

fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}
