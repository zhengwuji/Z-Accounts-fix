//! The captcha window: hosts the Aliyun captcha SDK (captcha.html) for the
//! claim flow. Tracks the (account, plan) pair awaiting captcha submission.

use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

static PENDING: Mutex<Option<(String, String)>> = Mutex::new(None);

pub fn set_pending(_app: &tauri::AppHandle, account_id: String, plan_id: String) {
    *PENDING.lock().unwrap() = Some((account_id, plan_id));
}

pub fn take_pending(_app: &tauri::AppHandle) -> Option<(String, String)> {
    PENDING.lock().unwrap().take()
}

pub fn open(app: &tauri::AppHandle, account_id: String, plan_id: String) -> Result<(), String> {
    set_pending(app, account_id, plan_id);
    if let Some(win) = app.get_webview_window("captcha") {
        let _ = win.show();
        let _ = win.set_focus();
        // pre-created window: reload so the captcha flow starts fresh each time
        let _ = win.reload();
        return Ok(());
    }
    // fallback: pre-created window is gone (only possible if creation failed
    // at startup) — build it on demand and show it right away
    WebviewWindowBuilder::new(app, "captcha", WebviewUrl::App("captcha.html".into()))
        .title("安全验证")
        .inner_size(420.0, 560.0)
        .resizable(false)
        .center()
        .additional_browser_args(
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --no-proxy-server",
        )
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
