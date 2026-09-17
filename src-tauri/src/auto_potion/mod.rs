//! Auto Potion Module
//! Monitors player HP/Mana in real-time and triggers user-configured potion keys
//! when levels drop below customized thresholds with cooldown protection.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use tauri::State;

use crate::offsets::{d2client, stat_list};
use crate::process::D2Context;
use crate::settings::{AutoPotionSettings, AutoPotionSlotConfig, AutoPotionTarget};

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerVitals {
    pub cur_hp: u32,
    pub max_hp: u32,
    pub hp_percent: u32,
    pub cur_mana: u32,
    pub max_mana: u32,
    pub mana_percent: u32,
}

pub struct AutoPotionState {
    pub settings: Mutex<AutoPotionSettings>,
    pub last_triggered: Mutex<HashMap<usize, Instant>>,
}

impl AutoPotionState {
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(AutoPotionSettings {
                enabled: true,
                slots: vec![
                    AutoPotionSlotConfig {
                        enabled: true,
                        target: AutoPotionTarget::Hp,
                        threshold_percent: 35,
                        key_code: 0x31,
                        key_display: "1".to_string(),
                        cooldown_ms: 600,
                    },
                    AutoPotionSlotConfig {
                        enabled: true,
                        target: AutoPotionTarget::Hp,
                        threshold_percent: 65,
                        key_code: 0x32,
                        key_display: "2".to_string(),
                        cooldown_ms: 600,
                    },
                    AutoPotionSlotConfig {
                        enabled: true,
                        target: AutoPotionTarget::Mana,
                        threshold_percent: 25,
                        key_code: 0x33,
                        key_display: "3".to_string(),
                        cooldown_ms: 600,
                    },
                ],
            }),
            last_triggered: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_settings(&self, new_settings: AutoPotionSettings) {
        if let Ok(mut settings) = self.settings.lock() {
            *settings = new_settings;
        }
    }
}

/// Reads the player's current and maximum HP & Mana directly from the unit's StatList
pub fn read_player_vitals(ctx: &D2Context) -> Option<PlayerVitals> {
    let p_player = match ctx
        .process
        .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let p_stat_list = match ctx
        .process
        .read_memory::<u32>(p_player + stat_list::UNIT_TO_STATS_LIST)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let p_stat = match ctx
        .process
        .read_memory::<u32>(p_stat_list + stat_list::SL_PSTAT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let count = (ctx
        .process
        .read_memory::<u16>(p_stat_list + stat_list::SL_STAT_COUNT)
        .unwrap_or(0) as usize)
        .min(256);

    let mut cur_hp: u32 = 0;
    let mut max_hp: u32 = 0;
    let mut cur_mana: u32 = 0;
    let mut max_mana: u32 = 0;

    for i in 0..count {
        let record = match p_stat.checked_add(i * stat_list::STAT_RECORD_SIZE) {
            Some(a) => a,
            None => break,
        };

        let nstat = ctx
            .process
            .read_memory::<u16>(record + stat_list::STAT_NSTAT)
            .unwrap_or(u16::MAX);
        let value = ctx
            .process
            .read_memory::<i32>(record + stat_list::STAT_VALUE)
            .unwrap_or(0);

        match nstat {
            6 => cur_hp = (value.max(0) >> 8) as u32,
            7 => max_hp = (value.max(0) >> 8) as u32,
            8 => cur_mana = (value.max(0) >> 8) as u32,
            9 => max_mana = (value.max(0) >> 8) as u32,
            _ => {}
        }
    }

    if max_hp == 0 {
        return None;
    }

    let hp_percent = ((cur_hp as u64 * 100) / max_hp as u64).min(100) as u32;
    let mana_percent = if max_mana > 0 {
        ((cur_mana as u64 * 100) / max_mana as u64).min(100) as u32
    } else {
        100
    };

    Some(PlayerVitals {
        cur_hp,
        max_hp,
        hp_percent,
        cur_mana,
        max_mana,
        mana_percent,
    })
}

#[cfg(target_os = "windows")]
fn get_game_hwnd() -> Option<windows::Win32::Foundation::HWND> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
    let class_name: Vec<u16> = "Diablo II\0".encode_utf16().collect();
    let hwnd = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) };
    if let Ok(h) = hwnd {
        if !h.0.is_null() {
            return Some(h);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn send_key_to_game(hwnd: windows::Win32::Foundation::HWND, vk: u32) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{MapVirtualKeyW, MAP_VIRTUAL_KEY_TYPE};
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_KEYDOWN, WM_KEYUP};

    let scan_code = unsafe { MapVirtualKeyW(vk, MAP_VIRTUAL_KEY_TYPE(0)) };
    let down_lparam = 1 | (scan_code << 16);
    let up_lparam = 1 | (scan_code << 16) | (1 << 30) | (1 << 31);

    unsafe {
        let _ = PostMessageW(
            hwnd,
            WM_KEYDOWN,
            WPARAM(vk as usize),
            LPARAM(down_lparam as isize),
        );
        let _ = PostMessageW(
            hwnd,
            WM_KEYUP,
            WPARAM(vk as usize),
            LPARAM(up_lparam as isize),
        );
    }
}

/// Evaluates player vitals against configured slots and triggers keystrokes if needed
pub fn tick_auto_potion(ctx: &D2Context, state: &AutoPotionState) -> Option<PlayerVitals> {
    let vitals = read_player_vitals(ctx)?;

    // Dead player does not drink potions
    if vitals.cur_hp == 0 {
        return Some(vitals);
    }

    let settings = match state.settings.lock() {
        Ok(s) => s.clone(),
        Err(_) => return Some(vitals),
    };

    if !settings.enabled {
        return Some(vitals);
    }

    #[cfg(target_os = "windows")]
    let hwnd = match get_game_hwnd() {
        Some(h) => h,
        None => return Some(vitals),
    };

    let now = Instant::now();
    let mut last_triggered = match state.last_triggered.lock() {
        Ok(guard) => guard,
        Err(_) => return Some(vitals),
    };

    for (idx, slot) in settings.slots.iter().enumerate() {
        if !slot.enabled || slot.key_code == 0 {
            continue;
        }

        let should_trigger = match slot.target {
            AutoPotionTarget::Hp => vitals.hp_percent <= slot.threshold_percent,
            AutoPotionTarget::Mana => vitals.mana_percent <= slot.threshold_percent,
        };

        if should_trigger {
            let cooldown = std::time::Duration::from_millis(slot.cooldown_ms as u64);
            let can_fire = match last_triggered.get(&idx) {
                Some(last_time) => now.duration_since(*last_time) >= cooldown,
                None => true,
            };

            if can_fire {
                #[cfg(target_os = "windows")]
                send_key_to_game(hwnd, slot.key_code);

                last_triggered.insert(idx, now);

                let target_name = match slot.target {
                    AutoPotionTarget::Hp => {
                        format!("HP {}% <= {}%", vitals.hp_percent, slot.threshold_percent)
                    }
                    AutoPotionTarget::Mana => format!(
                        "Mana {}% <= {}%",
                        vitals.mana_percent, slot.threshold_percent
                    ),
                };
                crate::logger::info(&format!(
                    "AutoPotion: Slot {} triggered ({}) -> Pressed key '{}' (0x{:X})",
                    idx + 1,
                    target_name,
                    slot.key_display,
                    slot.key_code
                ));
            }
        }
    }

    Some(vitals)
}

/// Tauri command to toggle auto potion overall or update slot settings
#[tauri::command]
pub fn update_auto_potion_settings(
    state: State<'_, AutoPotionState>,
    settings: AutoPotionSettings,
) -> Result<(), String> {
    state.update_settings(settings);
    crate::logger::info("AutoPotion: Settings updated from frontend");
    Ok(())
}
