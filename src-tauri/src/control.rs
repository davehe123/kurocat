use crate::cats::{all_cats, smoky_gray_tabby, upsert_custom_cat, CatSkinConfig};
use crate::{apply_breed_and_save, AppState};
use serde::Deserialize;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

pub fn show_control(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("control") {
        window.show()?;
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn open_controls(app: AppHandle) -> Result<(), String> {
    show_control(&app).map_err(|error| error.to_string())
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
