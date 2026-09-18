//! Item Extra Info Module
//! Directly hooks D2Sigma to display UID, CID, sockets and ethereal tags in item nameplates and tooltips.

pub mod hook;
pub use hook::ItemExtraInfoHook;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::hotkeys::HotkeyConfig;
use crate::logger::info as log_info;

#[cfg(target_os = "windows")]
use crate::hotkeys::chord_is_pressed_d2_only;
#[cfg(target_os = "linux")]
use crate::hotkeys::chord_is_pressed_d2_only_linux;

pub struct ItemExtraInfoState {
    pub hook: Mutex<ItemExtraInfoHook>,
}

impl ItemExtraInfoState {
    pub fn new() -> Self {
        Self {
            hook: Mutex::new(ItemExtraInfoHook::new()),
        }
    }
}

pub struct ItemExtraInfoHotkeyState {
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl ItemExtraInfoHotkeyState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            current_hotkey: Arc::new(Mutex::new(None)),
            thread_handle: Mutex::new(None),
        }
    }

    pub fn start(&self, app_handle: AppHandle, initial_hotkey: Option<HotkeyConfig>) {
        if let Ok(mut hk) = self.current_hotkey.lock() {
            *hk = initial_hotkey;
        }

        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let current_hotkey = self.current_hotkey.clone();

        #[cfg(target_os = "windows")]
        let handle = thread::spawn(move || {
            hotkey_thread_windows(is_running, current_hotkey, app_handle);
        });

        #[cfg(target_os = "linux")]
        let handle = thread::spawn(move || {
            hotkey_thread_linux(is_running, current_hotkey, app_handle);
        });

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let handle = thread::spawn(move || {
            let _ = (is_running, current_hotkey, app_handle);
        });

        if let Ok(mut th) = self.thread_handle.lock() {
            *th = Some(handle);
        }
    }

    pub fn update_hotkey(&self, new_hotkey: Option<HotkeyConfig>) {
        if let Ok(mut hk) = self.current_hotkey.lock() {
            *hk = new_hotkey;
        }
    }
}

#[cfg(target_os = "windows")]
fn hotkey_thread_windows(
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    app_handle: AppHandle,
) {
    let mut was_active = false;
    while is_running.load(Ordering::SeqCst) {
        let hk_opt = match current_hotkey.lock() {
            Ok(g) => g.clone(),
            Err(_) => break,
        };

        let active = if let Some(ref hk) = hk_opt {
            chord_is_pressed_d2_only(hk)
        } else {
            false
        };

        if active && !was_active {
            toggle_item_extra_info_internal(&app_handle);
        }
        was_active = active;
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "linux")]
fn hotkey_thread_linux(
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    app_handle: AppHandle,
) {
    let mut was_active = false;
    while is_running.load(Ordering::SeqCst) {
        let hk_opt = match current_hotkey.lock() {
            Ok(g) => g.clone(),
            Err(_) => break,
        };

        let active = if let Some(ref hk) = hk_opt {
            chord_is_pressed_d2_only_linux(hk)
        } else {
            false
        };

        if active && !was_active {
            toggle_item_extra_info_internal(&app_handle);
        }
        was_active = active;
        thread::sleep(Duration::from_millis(50));
    }
}

fn toggle_item_extra_info_internal(app_handle: &AppHandle) {
    if let Some(state) = app_handle.try_state::<ItemExtraInfoState>() {
        if let Ok(mut hook) = state.hook.lock() {
            let new_val = !hook.enabled;
            hook.enabled = new_val;
            log_info(&format!("ItemExtraInfo toggled via hotkey: {}", new_val));

            let _ = app_handle.emit("item-extra-info-toggled", new_val);
        }
    }
}

/// Tauri command to toggle the item extra info
#[tauri::command]
pub fn toggle_item_extra_info(
    state: tauri::State<'_, ItemExtraInfoState>,
    _app_state: tauri::State<'_, crate::AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut hook = state.hook.lock().map_err(|e| e.to_string())?;
    hook.enabled = enabled;
    Ok(())
}

/// Tauri command to toggle sockets and ethereal tags
#[tauri::command]
pub fn set_item_extra_info_show_sockets_and_eth(
    state: tauri::State<'_, ItemExtraInfoState>,
    _app_state: tauri::State<'_, crate::AppState>,
    show: bool,
) -> Result<(), String> {
    let mut hook = state.hook.lock().map_err(|e| e.to_string())?;
    hook.show_sockets_and_eth = show;
    Ok(())
}

/// Tauri command to update the hotkey for item extra info
#[tauri::command]
pub fn update_item_extra_info_hotkey(
    state: tauri::State<'_, ItemExtraInfoHotkeyState>,
    hotkey: Option<HotkeyConfig>,
) -> Result<(), String> {
    state.update_hotkey(hotkey);
    Ok(())
}
