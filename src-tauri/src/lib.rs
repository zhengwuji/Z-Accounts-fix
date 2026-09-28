//! App assembly: plugin setup, managed state, tray, background workers,
//! close-to-tray behaviour and the CLI dispatch entry point.

pub mod captcha_window;
pub mod claim;
pub mod cli;
pub mod commands;
pub mod crypto;
pub mod httpc;
pub mod i18n;
pub mod oauth;
pub mod quota;
pub mod store;
pub mod tray;
pub mod zcode;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use std::sync::{Arc, OnceLock};
use tauri::{Manager, WindowEvent};

static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

pub fn app_handle() -> Option<tauri::AppHandle> {
    APP.get().cloned()
}

pub fn b64url_nopad(data: &str) -> String {
    URL_SAFE_NO_PAD.encode(data.as_bytes())
}

pub fn run() {
    // CLI mode: `Z-Accounts.exe <subcommand> …` runs headless and exits.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(first) = args.first() {
        if !first.starts_with('-') {
            let code = cli::run(args);
            std::process::exit(code);
        }
    }

    let persist = match store::Persist::load() {
        Ok(p) => Arc::new(p),
        Err(e) => {
            eprintln!("fatal: cannot init data dir: {e}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // second instance → reveal the main window
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
                let _ = win.unminimize();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(persist.clone())
        .manage(oauth::shared())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::app_version,
            commands::get_recent_model_status,
            commands::set_language,
            commands::set_theme,
            commands::set_behavior,
            commands::set_auth_proxy,
            commands::capture_current,
            commands::rename_account,
            commands::delete_account,
            commands::update_account_from_live,
            commands::switch_to,
            commands::launch_zcode,
            commands::kill_zcode,
            commands::pick_zcode_path,
            commands::set_zcode_path,
            commands::open_external,
            commands::open_settings,
            commands::reveal_main,
            commands::autostart_status,
            commands::autostart_set,
            commands::get_account_quota,
            commands::claim_preview,
            commands::claim_refresh,
            commands::claim_captcha_config,
            commands::claim_start,
            commands::claim_cancel,
            commands::claim_captcha_submit,
            commands::export_pick_path,
            commands::export_finalize,
            commands::export_all_pick_path,
            commands::export_all_finalize,
            commands::import_pick_files,
            commands::import_sealed,
            commands::oauth_providers,
            commands::oauth_begin,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let _ = APP.set(handle.clone());

            // tray
            let p = persist.clone();
            tray::rebuild(&handle, &p).ok();

            // background: tray status refresh (ZCode start/stop, store changes)
            {
                let handle = handle.clone();
                let p = persist.clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    tray::rebuild(&handle, &p).ok();
                });
            }

            // background: auto-switch watchdog (2×exhausted → switch & restart)
            {
                let handle = handle.clone();
                let p = persist.clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(90));
                    let enabled = p.store.lock().unwrap().settings.auto_switch();
                    if enabled {
                        commands::auto::check_auto_switch(&handle, &p);
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let close_to_tray = app
                    .try_state::<Arc<store::Persist>>()
                    .map(|p| p.store.lock().unwrap().settings.close_to_tray())
                    .unwrap_or(true);
                if close_to_tray && window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Z-Accounts");
}
