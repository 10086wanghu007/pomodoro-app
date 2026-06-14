use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TimerMode {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub mode: TimerMode,
    pub time_left: u64,
    pub total_time: u64,
    pub is_running: bool,
    pub is_paused: bool,
    pub waiting_confirmation: bool,
    pub current_round: u32,
    pub completed_pomodoros: u32,
    pub total_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub work_time: f64,
    pub short_break: f64,
    pub long_break: f64,
    pub rounds: u32,
    #[serde(default = "default_force_popup")]
    pub force_popup: bool,
}

fn default_force_popup() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_time: 25.0,
            short_break: 5.0,
            long_break: 15.0,
            rounds: 4,
            force_popup: true,
        }
    }
}

struct Inner {
    state: TimerState,
    settings: Settings,
    last_tick: Option<Instant>,
}

pub struct PomodoroTimer {
    inner: Mutex<Inner>,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let settings = Settings::default();
        let time_left = (settings.work_time * 60.0 * 1000.0) as u64;
        Self {
            inner: Mutex::new(Inner {
                state: TimerState {
                    mode: TimerMode::Work,
                    time_left,
                    total_time: time_left,
                    is_running: false,
                    is_paused: false,
                    waiting_confirmation: false,
                    current_round: 1,
                    completed_pomodoros: 0,
                    total_minutes: 0,
                },
                settings,
                last_tick: None,
            }),
        }
    }

    pub fn start(&self) {
        let mut inner = self.inner.lock().unwrap();
        // 如果正在等待确认，不允许开始
        if inner.state.waiting_confirmation {
            return;
        }
        if inner.state.is_paused {
            inner.state.is_running = true;
            inner.state.is_paused = false;
            inner.last_tick = Some(Instant::now());
            return;
        }
        let time_left = (inner.settings.work_time * 60.0 * 1000.0) as u64;
        let completed = inner.state.completed_pomodoros;
        let total_min = inner.state.total_minutes;
        inner.state = TimerState {
            mode: TimerMode::Work,
            time_left,
            total_time: time_left,
            is_running: true,
            is_paused: false,
            waiting_confirmation: false,
            current_round: 1,
            completed_pomodoros: completed,
            total_minutes: total_min,
        };
        inner.last_tick = Some(Instant::now());
    }

    pub fn pause(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.state.is_running = false;
        inner.state.is_paused = true;
        if let Some(last) = inner.last_tick {
            let elapsed_ms = last.elapsed().as_millis() as u64;
            inner.state.time_left = inner.state.time_left.saturating_sub(elapsed_ms);
        }
        inner.last_tick = None;
    }

    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        let time_left = (inner.settings.work_time * 60.0 * 1000.0) as u64;
        let completed = inner.state.completed_pomodoros;
        let total_min = inner.state.total_minutes;
        inner.state = TimerState {
            mode: TimerMode::Work,
            time_left,
            total_time: time_left,
            is_running: false,
            is_paused: false,
            waiting_confirmation: false,
            current_round: 1,
            completed_pomodoros: completed,
            total_minutes: total_min,
        };
        inner.last_tick = None;
    }

    pub fn tick(&self) -> Option<TimerEvent> {
        let mut inner = self.inner.lock().unwrap();
        if !inner.state.is_running {
            return None;
        }
        let Some(last) = inner.last_tick else {
            return None;
        };
        let elapsed_ms = last.elapsed().as_millis() as u64;
        if elapsed_ms < 50 {
            return None;
        }
        inner.last_tick = Some(Instant::now());

        if elapsed_ms >= inner.state.time_left {
            inner.state.time_left = 0;
            return Some(Self::complete_phase(&mut inner));
        }
        inner.state.time_left -= elapsed_ms;
        None
    }

    fn complete_phase(inner: &mut Inner) -> TimerEvent {
        let s = inner.settings.clone();
        match inner.state.mode {
            TimerMode::Work => {
                inner.state.completed_pomodoros += 1;
                inner.state.total_minutes += s.work_time as u64;
                // 工作完成，等待用户确认进入休息
                inner.state.is_running = false;
                inner.state.waiting_confirmation = true;
                inner.last_tick = None;

                if inner.state.current_round >= s.rounds {
                    TimerEvent::WorkComplete { go_to: "longBreak".into() }
                } else {
                    TimerEvent::WorkComplete { go_to: "shortBreak".into() }
                }
            }
            TimerMode::ShortBreak | TimerMode::LongBreak => {
                // 休息完成，等待用户确认进入工作
                inner.state.current_round += 1;
                inner.state.is_running = false;
                inner.state.waiting_confirmation = true;
                inner.last_tick = None;
                TimerEvent::BreakComplete
            }
        }
    }

    pub fn confirm_next_phase(&self) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.state.waiting_confirmation {
            return;
        }
        let s = inner.settings.clone();
        match inner.state.mode {
            TimerMode::Work => {
                // 工作完成，确认进入休息
                if inner.state.current_round >= s.rounds {
                    inner.state.mode = TimerMode::LongBreak;
                    inner.state.time_left = (s.long_break * 60.0 * 1000.0) as u64;
                } else {
                    inner.state.mode = TimerMode::ShortBreak;
                    inner.state.time_left = (s.short_break * 60.0 * 1000.0) as u64;
                }
                inner.state.total_time = inner.state.time_left;
            }
            TimerMode::ShortBreak | TimerMode::LongBreak => {
                // 休息完成，确认进入工作
                inner.state.mode = TimerMode::Work;
                inner.state.time_left = (s.work_time * 60.0 * 1000.0) as u64;
                inner.state.total_time = inner.state.time_left;
            }
        }
        inner.state.waiting_confirmation = false;
        inner.state.is_running = true;
        inner.last_tick = Some(Instant::now());
    }

    pub fn get_state(&self) -> TimerState {
        let inner = self.inner.lock().unwrap();
        if inner.state.is_running {
            if let Some(last) = inner.last_tick {
                let elapsed_ms = last.elapsed().as_millis() as u64;
                if elapsed_ms < inner.state.time_left {
                    let mut snapshot = inner.state.clone();
                    snapshot.time_left = inner.state.time_left - elapsed_ms;
                    return snapshot;
                }
            }
        }
        inner.state.clone()
    }

    pub fn update_settings(&self, new_settings: Settings) {
        self.inner.lock().unwrap().settings = new_settings;
    }

    pub fn get_settings(&self) -> Settings {
        self.inner.lock().unwrap().settings.clone()
    }

    pub fn extend_time(&self, minutes: f64) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state.is_running || inner.state.is_paused || inner.state.waiting_confirmation {
            let additional_ms = (minutes * 60.0 * 1000.0) as u64;
            inner.state.time_left += additional_ms;
            inner.state.total_time += additional_ms;
            // 如果正在等待确认，取消等待状态，继续计时
            if inner.state.waiting_confirmation {
                inner.state.waiting_confirmation = false;
                inner.state.is_running = true;
                inner.last_tick = Some(Instant::now());
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum TimerEvent {
    WorkComplete { go_to: String },
    BreakComplete,
}
