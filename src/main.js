const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// DOM elements
const els = {
    timerDisplay: document.getElementById('timer-display'),
    timerLabel: document.getElementById('timer-label'),
    startBtn: document.getElementById('start-btn'),
    pauseBtn: document.getElementById('pause-btn'),
    resetBtn: document.getElementById('reset-btn'),
    currentRound: document.getElementById('current-round'),
    totalRounds: document.getElementById('total-rounds'),
    status: document.getElementById('status'),
    workTime: document.getElementById('work-time'),
    shortBreak: document.getElementById('short-break'),
    longBreak: document.getElementById('long-break'),
    rounds: document.getElementById('rounds'),
    forcePopup: document.getElementById('force-popup'),
    saveSettings: document.getElementById('save-settings'),
    completedCount: document.getElementById('completed-count'),
    totalMinutes: document.getElementById('total-minutes'),
    timerRing: document.getElementById('timer-ring'),
};

const circumference = 2 * Math.PI * 90; // r=90
const modeLabels = { work: '工作时间', shortBreak: '短休息', longBreak: '长休息' };
const modeClasses = { work: 'mode-work', shortBreak: 'mode-shortBreak', longBreak: 'mode-longBreak' };

let currentMode = 'work';
let currentTotalTime = 1500;

function formatTime(ms) {
    const totalSeconds = Math.ceil(ms / 1000);
    const m = Math.floor(totalSeconds / 60);
    const s = totalSeconds % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

function updateProgress(timeLeft, totalTime) {
    const progress = (timeLeft / totalTime) * circumference;
    const progressEl = document.querySelector('.timer-progress');
    if (progressEl) {
        progressEl.style.strokeDashoffset = circumference - progress;
    }
}

function updateModeColors(mode) {
    document.body.className = modeClasses[mode] || '';
}

function applyState(state) {
    els.timerDisplay.textContent = formatTime(state.timeLeft);
    els.timerLabel.textContent = modeLabels[state.mode] || state.mode;
    els.currentRound.textContent = state.currentRound;
    els.status.textContent = state.waitingConfirmation
        ? '等待确认'
        : state.isRunning
            ? (state.mode === 'work' ? '专注中...' : '休息中...')
            : state.isPaused ? '已暂停' : '准备开始';

    els.startBtn.disabled = state.isRunning;
    els.pauseBtn.disabled = !state.isRunning && !state.isPaused;
    els.resetBtn.disabled = !state.isRunning && !state.isPaused && !state.waitingConfirmation;

    if (state.isRunning) {
        els.startBtn.textContent = '运行中';
    } else if (state.isPaused) {
        els.startBtn.textContent = '继续';
    } else if (state.waitingConfirmation) {
        // 根据当前模式显示不同的确认文字
        els.startBtn.textContent = state.mode === 'work' ? '开始休息' : '开始工作';
    } else {
        els.startBtn.textContent = '开始';
    }

    if (state.mode !== currentMode) {
        currentMode = state.mode;
        updateModeColors(currentMode);
    }

    currentTotalTime = state.totalTime;
    updateProgress(state.timeLeft, state.totalTime);

    els.completedCount.textContent = state.completedPomodoros;
    els.totalMinutes.textContent = state.totalMinutes;

    if (state.isRunning) {
        els.timerRing.classList.add('active');
    } else {
        els.timerRing.classList.remove('active');
    }
}

async function init() {
    // Load config from backend
    try {
        const config = await invoke('load_config');
        if (config.settings) {
            els.workTime.value = config.settings.workTime;
            els.shortBreak.value = config.settings.shortBreak;
            els.longBreak.value = config.settings.longBreak;
            els.rounds.value = config.settings.rounds;
            els.totalRounds.textContent = config.settings.rounds;
            els.forcePopup.checked = config.settings.forcePopup !== false;
        }
    } catch (e) {
        console.error('Failed to load config:', e);
    }

    // Get initial state
    try {
        const state = await invoke('get_timer_state');
        applyState(state);
    } catch (e) {
        console.error('Failed to get state:', e);
    }

    // Listen for state updates from Rust backend
    await listen('timer-state', (event) => {
        applyState(event.payload);
    });

    // 精美弹窗提醒
    const modalOverlay = document.getElementById('modal-overlay');
    const modalCard = document.getElementById('modal-card');
    const modalTitle = document.getElementById('modal-title');
    const modalSubtitle = document.getElementById('modal-subtitle');
    const modalConfirm = document.getElementById('modal-confirm');
    const modalSnooze = document.getElementById('modal-snooze');
    const modalProgressBar = document.getElementById('modal-progress-bar');

    let modalTimer = null;

    function showModal(title, subtitle, showSnooze = true) {
        modalTitle.textContent = title;
        modalSubtitle.textContent = subtitle;
        modalSnooze.style.display = showSnooze ? 'block' : 'none';

        // 重置动画
        modalCard.style.animation = 'none';
        modalCard.offsetHeight; // 触发重绘
        modalCard.style.animation = '';

        modalProgressBar.style.animation = 'none';
        modalProgressBar.offsetHeight;
        modalProgressBar.style.animation = '';

        modalOverlay.classList.add('show');

        // 8秒后自动关闭
        if (modalTimer) clearTimeout(modalTimer);
        modalTimer = setTimeout(() => {
            hideModal();
        }, 8000);
    }

    function hideModal() {
        modalOverlay.classList.remove('show');
        if (modalTimer) {
            clearTimeout(modalTimer);
            modalTimer = null;
        }
    }

    // 按钮事件
    modalConfirm.addEventListener('click', async () => {
        hideModal();
        // 确认进入下一阶段
        try {
            await invoke('confirm_next_phase');
        } catch (e) {
            console.error('Failed to confirm next phase:', e);
        }
    });
    modalSnooze.addEventListener('click', async () => {
        hideModal();
        // 延长5分钟工作时间
        try {
            await invoke('extend_timer', { minutes: 5.0 });
        } catch (e) {
            console.error('Failed to extend timer:', e);
        }
    });

    // 点击遮罩关闭
    modalOverlay.addEventListener('click', (e) => {
        if (e.target === modalOverlay) {
            hideModal();
        }
    });

    // ESC 键关闭
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && modalOverlay.classList.contains('show')) {
            hideModal();
        }
    });

    await listen('timer-event', (event) => {
        const evt = event.payload;
        // Flash the timer ring
        els.timerRing.classList.add('flash');
        setTimeout(() => els.timerRing.classList.remove('flash'), 2000);

        // Show beautiful modal
        if (evt.type === 'WorkComplete') {
            showModal('时间到', '休息一下吧，你辛苦了', true);
        } else {
            showModal('休息结束', '准备好开始新的专注了吗？', false);
        }
    });

    // Button handlers
    els.startBtn.addEventListener('click', async () => {
        try {
            const state = await invoke('get_timer_state');
            if (state.waitingConfirmation) {
                await invoke('confirm_next_phase');
            } else {
                await invoke('start_timer');
            }
        } catch (e) {
            console.error('Failed to start/confirm:', e);
        }
    });
    els.pauseBtn.addEventListener('click', () => invoke('pause_timer'));
    els.resetBtn.addEventListener('click', () => invoke('reset_timer'));

    els.saveSettings.addEventListener('click', async () => {
        const settings = {
            workTime: parseFloat(els.workTime.value) || 25,
            shortBreak: parseFloat(els.shortBreak.value) || 5,
            longBreak: parseFloat(els.longBreak.value) || 15,
            rounds: parseInt(els.rounds.value) || 4,
            forcePopup: els.forcePopup.checked,
        };
        els.totalRounds.textContent = settings.rounds;
        try {
            await invoke('save_settings', { settings });
        } catch (e) {
            console.error('Failed to save settings:', e);
        }
    });
}

window.addEventListener('DOMContentLoaded', init);
