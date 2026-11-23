use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static CONTROL_READY: AtomicBool = AtomicBool::new(false);
static CONTROL_SHOWN: AtomicBool = AtomicBool::new(false);

pub fn initialize() {
    let preferred = crate::config::app_data_dir().join("startup.log");
    let fallback = std::env::temp_dir().join("tuanzi-cat-startup.log");
    for path in [preferred, fallback] {
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                continue;
            }
        }
        if OpenOptions::new().create(true).append(true).open(&path).is_ok() {
            let _ = LOG_PATH.set(path);
            break;
        }
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log(format!("panic: {info}"));
        previous(info);
    }));
    log("launch");
}

pub fn log(message: impl AsRef<str>) {
    let Some(path) = LOG_PATH.get() else { return };
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{timestamp} PID={} {}", std::process::id(), message.as_ref());
    }
}

pub fn frontend_ready() {
    log("control frontend ready");
    CONTROL_READY.store(true, Ordering::Release);
}

pub fn control_shown() {
    log("control window shown");
    CONTROL_SHOWN.store(true, Ordering::Release);
}

pub fn wait_for_smoke_check(app: AppHandle) {
    if !std::env::args().any(|arg| arg == "--startup-check") {
        return;
    }
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if CONTROL_READY.load(Ordering::Acquire) && CONTROL_SHOWN.load(Ordering::Acquire) {
                log("startup check passed");
                app.exit(0);
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        log("startup check failed: control frontend did not report ready");
        app.exit(1);
    });
}

pub fn report_failure(error: &str) {
    log(format!("startup failed: {error}"));
    #[cfg(windows)]
    if !std::env::args().any(|arg| arg == "--startup-check") {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let path = LOG_PATH.get().map(|path| path.display().to_string())
            .unwrap_or_else(|| "%APPDATA%\\tuanzi-cat\\startup.log".into());
        let heading: Vec<u16> = "团子桌宠启动失败\0".encode_utf16().collect();
        let text: Vec<u16> = format!("{error}\n\n启动日志\n{path}\0").encode_utf16().collect();
        unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), heading.as_ptr(), MB_OK | MB_ICONERROR); }
    }
}
