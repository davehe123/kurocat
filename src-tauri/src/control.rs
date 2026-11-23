use crate::cats::{all_cats, smoky_gray_tabby, upsert_custom_cat, CatSkinConfig};
use crate::{apply_breed_and_save, AppState};
use serde::Deserialize;
use std::sync::{atomic::Ordering, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

static CONTROL_WINDOW: Mutex<()> = Mutex::new(());

fn show_control(app: &AppHandle) -> Result<(), String> {
    let _guard = CONTROL_WINDOW.lock().map_err(|error| error.to_string())?;
    let window = match app.get_webview_window("control") {
        Some(window) => window,
        None => {
            let config = app.config().app.windows.iter()
                .find(|window| window.label == "control")
                .ok_or("缺少控制面板配置。")?;
            tauri::WebviewWindowBuilder::from_config(app, config)
                .and_then(|builder| builder.build())
                .map_err(|error| error.to_string())?
        }
    };
    window.show().map_err(|error| error.to_string())?;
    if window.is_minimized().unwrap_or(false) {
        let _ = window.unminimize();
    }
    let _ = window.set_focus();
    if !window.is_visible().unwrap_or(false) {
        return Err("控制面板窗口没有显示。".into());
    }
    crate::startup::control_shown();
    Ok(())
}

pub fn schedule_control(app: AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = show_control(&app) {
            crate::startup::report_failure(&error);
        }
    });
}

#[tauri::command]
pub async fn open_controls(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || show_control(&app))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn control_ready(window: tauri::WebviewWindow) {
    if window.label() == "control" {
        crate::startup::frontend_ready();
    }
}

#[tauri::command]
pub fn show_desktop_pet(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.paused_guard.load(Ordering::Relaxed) {
        return Err("键鼠监听已从托盘暂停。请在托盘选择「显示 / 隐藏桌宠」恢复。".into());
    }
    let window = app.get_webview_window("main").ok_or("找不到桌宠窗口")?;
    window.show().map_err(|error| error.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinDraft {
    name: String,
    fur: String,
    eyes: String,
    accent: String,
}

fn hex_color(value: &str) -> Result<String, String> {
    let hex = value.strip_prefix('#').ok_or("颜色格式应为 #RRGGBB")?;
    if hex.len() != 6 || !hex.bytes().all(|digit| digit.is_ascii_hexdigit()) {
        return Err("颜色格式应为 #RRGGBB".into());
    }
    Ok(value.to_ascii_lowercase())
}

#[tauri::command]
pub fn save_custom_skin(app: AppHandle, draft: SkinDraft) -> Result<CatSkinConfig, String> {
    let name = draft.name.trim();
    if name.is_empty() || name.chars().count() > 32 || name.chars().any(char::is_control) {
        return Err("请输入 1 至 32 个字符的角色名字。".into());
    }
    let fur = hex_color(&draft.fur)?;
    let eyes = hex_color(&draft.eyes)?;
    let accent = hex_color(&draft.accent)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let mut skin = smoky_gray_tabby();
    skin.id = format!("tuanzi-{:x}", stamp);
    skin.name = name.to_string();
    skin.builtin = false;
    skin.fur_color = fur.clone();
    skin.fur_secondary_color = None;
    skin.arm_color = Some(fur);
    skin.paw_color = None;
    skin.eye_color_top = eyes.clone();
    skin.eye_color_bottom = Some(eyes);
    skin.accent_color = Some(accent);
    let saved = upsert_custom_cat(skin)?;
    let _ = app.emit("kuro-cats", all_cats());
    apply_breed_and_save(&app, &saved.id);
    Ok(saved)
}
