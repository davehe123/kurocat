use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_cat_breed() -> String {
    "smoky-gray-tabby".to_string()
}

fn default_mcp_enabled() -> bool {
    true
}

fn default_privacy_password_guard() -> bool {
    true
}

fn default_mcp_port() -> u16 {
    27421
}

pub fn default_keyboard_layout() -> String {
    if cfg!(feature = "layout-108") {
        "108".to_string()
    } else {
        "65".to_string()
    }
}

pub fn normalize_keyboard_layout(raw: &str) -> String {
    if raw.trim() == "108" {
        "108".to_string()
    } else {
        "65".to_string()
    }
}

pub fn default_keyboard_orientation() -> String {
    "cat".to_string()
}

pub fn normalize_keyboard_orientation(raw: &str) -> String {
    if raw.trim() == "viewer" {
        "viewer".to_string()
    } else {
        "cat".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub scale: f64,
    pub always_on_top: bool,
    pub click_through_locked: bool,
    pub show_key_bubbles: bool,
    pub sound_enabled: bool,
    pub autostart: bool,
    #[serde(default = "default_cat_breed")]
    pub cat_breed: String,
    #[serde(default = "default_mcp_enabled")]
    pub mcp_enabled: bool,
    #[serde(default = "default_mcp_port")]
    pub mcp_port: u16,
    #[serde(default = "default_keyboard_layout")]
    pub keyboard_layout: String,
    #[serde(default = "default_keyboard_orientation")]
    pub keyboard_orientation: String,
    #[serde(default = "default_privacy_password_guard")]
    pub privacy_password_guard: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    pub scale: Option<f64>,
    pub always_on_top: Option<bool>,
    pub click_through_locked: Option<bool>,
    pub show_key_bubbles: Option<bool>,
    pub sound_enabled: Option<bool>,
    pub autostart: Option<bool>,
    pub cat_breed: Option<String>,
    pub mcp_enabled: Option<bool>,
    pub mcp_port: Option<u16>,
    pub keyboard_layout: Option<String>,
    pub keyboard_orientation: Option<String>,
    pub privacy_password_guard: Option<bool>,
}

impl SettingsPatch {
    pub fn apply(self, settings: &mut AppSettings) {
        macro_rules! write_if_present {
            ($($field:ident),+ $(,)?) => {
                $(if let Some(value) = self.$field {
                    settings.$field = value;
                })+
            };
        }
        write_if_present!(scale, always_on_top, click_through_locked, show_key_bubbles, sound_enabled, autostart, cat_breed, mcp_enabled, mcp_port, keyboard_layout, keyboard_orientation, privacy_password_guard);
    }
}

#[cfg(test)]
mod settings_patch_tests {
    use super::*;

    #[test]
    fn a_partial_patch_preserves_unrelated_settings_and_position() {
        let mut settings = AppSettings {
            x: Some(-800),
            y: Some(200),
            cat_breed: "ragdoll".into(),
            show_key_bubbles: false,
            ..AppSettings::default()
        };
        let patch: SettingsPatch = serde_json::from_str(r#"{"scale":0.85}"#).unwrap();
        patch.apply(&mut settings);
        assert_eq!(settings.scale, 0.85);
        assert_eq!((settings.x, settings.y), (Some(-800), Some(200)));
        assert_eq!(settings.cat_breed, "ragdoll");
        assert!(!settings.show_key_bubbles);
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            scale: 1.0,
            always_on_top: true,
            click_through_locked: false,
            show_key_bubbles: true,
            sound_enabled: true,
            autostart: false,
            cat_breed: default_cat_breed(),
            mcp_enabled: default_mcp_enabled(),
            mcp_port: default_mcp_port(),
            keyboard_layout: default_keyboard_layout(),
            keyboard_orientation: default_keyboard_orientation(),
            privacy_password_guard: default_privacy_password_guard(),
        }
    }
}

pub fn app_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        base.join("tuanzi-cat")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        home.join("Library")
            .join("Application Support")
            .join("tuanzi-cat")
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        home.join(".config").join("tuanzi-cat")
    }
}

pub fn config_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut parsed) = serde_json::from_str::<AppSettings>(&content) {
            parsed.scale = parsed.scale.clamp(0.20, 1.50);
            return parsed;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

#[cfg(windows)]
pub fn set_windows_autostart(enabled: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = to_wide("TuanziCat");

    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if status != 0 {
            return Err(format!("RegOpenKeyExW failed with code {}", status));
        }

        let result = if enabled {
            let exe = std::env::current_exe()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            let quoted = format!("\"{}\" --background", exe);
            let data = to_wide(&quoted);
            let byte_len = (data.len() * std::mem::size_of::<u16>()) as u32;
            let st = RegSetValueExW(
                hkey,
                value_name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr() as *const u8,
                byte_len,
            );
            if st == 0 {
                Ok(())
            } else {
                Err(format!("RegSetValueExW failed with code {}", st))
            }
        } else {
            let _ = RegDeleteValueW(hkey, value_name.as_ptr());
            Ok(())
        };

        RegCloseKey(hkey);
        result
    }
}

#[cfg(target_os = "macos")]
pub fn set_windows_autostart(enabled: bool) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let agents_dir = PathBuf::from(home).join("Library").join("LaunchAgents");
    let plist_path = agents_dir.join("com.tuanzi.cat.plist");

    if enabled {
        fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.tuanzi.cat</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--background</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
            exe
        );
        fs::write(plist_path, plist).map_err(|e| e.to_string())
    } else {
        if plist_path.exists() {
            let _ = fs::remove_file(plist_path);
        }
        Ok(())
    }
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn set_windows_autostart(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub fn create_desktop_shortcut() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe_path.to_string_lossy().replace('\'', "''");
    let work_dir = exe_path
        .parent()
        .map(|p| p.to_string_lossy().replace('\'', "''"))
        .unwrap_or_default();

    let script = format!(
        "$desktop = [Environment]::GetFolderPath('Desktop'); \
         $lnk = Join-Path $desktop 'TuanziCat.lnk'; \
         $ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut($lnk); \
         $s.TargetPath = '{exe}'; \
         $s.WorkingDirectory = '{wd}'; \
         $s.IconLocation = '{exe},0'; \
         $s.Description = 'Tuanzi Cat Desktop Companion'; \
         $s.Save(); \
         Write-Output $lnk",
        exe = exe_str,
        wd = work_dir,
    );

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to invoke PowerShell: {}", e))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(path)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Failed to create desktop shortcut".to_string()
        } else {
            err
        })
    }
}

#[cfg(not(windows))]
pub fn create_desktop_shortcut() -> Result<String, String> {
    Ok("Desktop shortcut is managed via Applications on this OS".to_string())
}
