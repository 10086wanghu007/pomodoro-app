use crate::timer::{Settings, TimerState};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn config_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    std::fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfig {
    settings: Option<Settings>,
    completed: Option<u32>,
    total_minutes: Option<u64>,
}

#[tauri::command]
pub fn get_timer_state(timer: tauri::State<'_, crate::timer::PomodoroTimer>) -> TimerState {
    timer.get_state()
}

#[tauri::command]
pub fn start_timer(timer: tauri::State<'_, crate::timer::PomodoroTimer>) {
    timer.start();
}

#[tauri::command]
pub fn pause_timer(timer: tauri::State<'_, crate::timer::PomodoroTimer>) {
    timer.pause();
}

#[tauri::command]
pub fn reset_timer(timer: tauri::State<'_, crate::timer::PomodoroTimer>) {
    timer.reset();
}

#[tauri::command]
pub fn get_settings(timer: tauri::State<'_, crate::timer::PomodoroTimer>) -> Settings {
    timer.get_settings()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    timer: tauri::State<'_, crate::timer::PomodoroTimer>,
    settings: Settings,
) -> Result<(), String> {
    timer.update_settings(settings.clone());
    let path = config_path(&app);
    let config = AppConfig {
        settings: Some(settings),
        ..Default::default()
    };
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_config(
    app: AppHandle,
    timer: tauri::State<'_, crate::timer::PomodoroTimer>,
) -> Result<AppConfig, String> {
    let path = config_path(&app);
    if path.exists() {
        let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let config: AppConfig = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        if let Some(ref s) = config.settings {
            timer.update_settings(s.clone());
        }
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
pub fn send_notification(app: AppHandle, title: String, body: String) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

#[tauri::command]
pub fn extend_timer(timer: tauri::State<'_, crate::timer::PomodoroTimer>, minutes: f64) {
    timer.extend_time(minutes);
}

#[tauri::command]
pub fn confirm_next_phase(timer: tauri::State<'_, crate::timer::PomodoroTimer>) {
    timer.confirm_next_phase();
}
