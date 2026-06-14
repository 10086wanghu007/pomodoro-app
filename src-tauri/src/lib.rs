mod commands;
pub mod timer;

use tauri::{Emitter, Manager};
use timer::PomodoroTimer;

#[cfg(windows)]
extern "system" {
    fn MessageBeep(uType: u32) -> i32;
}

/// Play a system notification sound (cross-platform)
fn play_alarm() {
    #[cfg(windows)]
    unsafe {
        // 0xFFFFFFFF = default beep, 0x00000040 = MB_ICONASTERISK (info sound)
        MessageBeep(0x00000040);
        // Play a second beep after a short pause for emphasis
        std::thread::sleep(std::time::Duration::from_millis(300));
        MessageBeep(0x00000040);
    }
}

/// 强制显示窗口到最前面
fn show_window_to_front(handle: &tauri::AppHandle) {
    if let Some(window) = handle.get_webview_window("main") {
        // 取消最小化
        let _ = window.unminimize();
        // 显示窗口
        let _ = window.show();
        // 设置置顶
        let _ = window.set_always_on_top(true);
        // 设置焦点
        let _ = window.set_focus();
        // 任务栏闪烁（Windows）
        #[cfg(windows)]
        {
            let _ = window.request_user_attention(Some(tauri::UserAttentionType::Critical));
        }
        // 3秒后取消置顶，避免影响正常使用
        let window_clone = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = window_clone.set_always_on_top(false);
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(PomodoroTimer::new())
        .setup(|app| {
            let handle = app.handle().clone();
            // Background tick loop: polls every 200ms, emits state + events
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    let timer = handle.state::<PomodoroTimer>();
                    if let Some(event) = timer.tick() {
                        // Send system notification directly from Rust
                        let (title, body) = match &event {
                            timer::TimerEvent::WorkComplete { go_to } => {
                                if go_to == "longBreak" {
                                    ("番茄钟", "太棒了！休息一下吧")
                                } else {
                                    ("番茄钟", "休息一下吧")
                                }
                            }
                            timer::TimerEvent::BreakComplete => ("番茄钟", "开始工作吧"),
                        };
                        use tauri_plugin_notification::NotificationExt;
                        let _ = handle
                            .notification()
                            .builder()
                            .title(title)
                            .body(body)
                            .show();

                        // Play alarm sound
                        play_alarm();

                        // 根据设置决定是否强制弹窗
                        let settings = timer.get_settings();
                        if settings.force_popup {
                            show_window_to_front(&handle);
                        }

                        // Also emit to frontend for UI updates
                        let _ = handle.emit("timer-event", &event);
                    }
                    let state = timer.get_state();
                    let _ = handle.emit("timer-state", &state);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_timer_state,
            commands::start_timer,
            commands::pause_timer,
            commands::reset_timer,
            commands::get_settings,
            commands::save_settings,
            commands::load_config,
            commands::send_notification,
            commands::extend_timer,
            commands::confirm_next_phase,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
