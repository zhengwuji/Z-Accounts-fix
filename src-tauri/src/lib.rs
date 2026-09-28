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

            // Pre-create the settings & captcha windows NOW (event loop not
            // running yet, same path as the main window). Creating a webview
            // window later from an IPC command deadlocks: wry's WebView2
            // completion callbacks are awaited with a GetMessage pump that
            // re-enters the tao event loop, so the new webview never navigates
            // (white window) and the caller never returns. These windows are
            // hidden on close instead of destroyed (see on_window_event).
            let _ = tauri::WebviewWindowBuilder::new(
                &handle,
                "settings",
                tauri::WebviewUrl::App("settings.html".into()),
            )
            .title("Z-Accounts 设置")
            .inner_size(720.0, 780.0)
            .min_inner_size(620.0, 560.0)
            .visible(false)
            .resizable(true)
            .center()
            .additional_browser_args(
                "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --no-proxy-server",
            )
            .build();
            let _ = tauri::WebviewWindowBuilder::new(
                &handle,
                "captcha",
                tauri::WebviewUrl::App("captcha.html".into()),
            )
            .title("安全验证")
            .inner_size(420.0, 560.0)
            .visible(false)
            .resizable(false)
            .center()
            // must match the other windows' browser args exactly — WebView2
            // rejects an environment whose args differ within the same
            // user data folder
            .additional_browser_args(
                "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --no-proxy-server",
            )
            .build()
            .inspect_err(|e| eprintln!("pre-create captcha window failed: {e}"));

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
                match window.label() {
                    // pre-created windows are only hidden, never destroyed
                    // (recreating them dynamically would deadlock, see setup)
                    "settings" | "captcha" => {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    "main" => {
                        let close_to_tray = app
                            .try_state::<Arc<store::Persist>>()
                            .map(|p| p.store.lock().unwrap().settings.close_to_tray())
                            .unwrap_or(true);
                        if close_to_tray {
                            let _ = window.hide();
                            api.prevent_close();
                        }
                    }
                    _ => {}
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Z-Accounts");
}
