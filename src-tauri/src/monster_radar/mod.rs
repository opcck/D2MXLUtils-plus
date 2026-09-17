//! Monster Minimap Radar Module
//! Displays normal, champion/unique and boss monsters on the automap using D2Sigma native drawing routines.

pub mod boss_data;
pub mod hook;

pub use hook::MonsterRadarHook;
use std::sync::Mutex;

pub struct MonsterRadarState {
    pub hook: Mutex<MonsterRadarHook>,
}

impl MonsterRadarState {
    pub fn new() -> Self {
        Self {
            hook: Mutex::new(MonsterRadarHook::new()),
        }
    }
}

/// Tauri command to toggle the monster minimap radar
#[tauri::command]
pub fn toggle_monster_radar(
    state: tauri::State<'_, MonsterRadarState>,
    app_state: tauri::State<'_, crate::AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut hook = state.hook.lock().map_err(|e| e.to_string())?;
    hook.enabled = enabled;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    if let Ok(guard) = app_state.scanner_shared_state.read() {
        if let Some(shared) = guard.as_ref() {
            if hook.is_injected() {
                hook.set_enabled(&shared.ctx.process, enabled)?;
            } else if enabled {
                hook.inject(&shared.ctx.process, shared.ctx.d2_sigma)?;
            }
        }
    }
    Ok(())
}

/// Tauri command to toggle normal monster display on the radar
#[tauri::command]
pub fn set_radar_show_normal(
    state: tauri::State<'_, MonsterRadarState>,
    app_state: tauri::State<'_, crate::AppState>,
    show_normal: bool,
) -> Result<(), String> {
    let mut hook = state.hook.lock().map_err(|e| e.to_string())?;
    hook.show_normal = show_normal;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    if let Ok(guard) = app_state.scanner_shared_state.read() {
        if let Some(shared) = guard.as_ref() {
            if hook.is_injected() {
                hook.set_show_normal(&shared.ctx.process, show_normal)?;
            }
        }
    }
    Ok(())
}
