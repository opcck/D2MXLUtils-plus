//! Combat Tweaks Module
//! Provides continuous attack QoL: when holding mouse button to attack/cast,
//! the character continues attacking towards the mouse cursor even after the
//! current target monster dies.

use crate::offsets::d2client::{
    CONTINUOUS_ATTACK_ORIGINAL, CONTINUOUS_ATTACK_PATCH, CONTINUOUS_ATTACK_PATCHED,
};
use crate::process::ProcessHandle;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::FlushInstructionCache;
#[cfg(target_os = "windows")]
use windows::Win32::System::Memory::{
    VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

#[cfg(target_os = "windows")]
fn patch_remote_byte(process: &ProcessHandle, addr: usize, byte_val: u8) -> Result<(), String> {
    let bytes = [byte_val];
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            1,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
        .map_err(|e| format!("VirtualProtectEx RWX failed: {}", e))?;
    }
    process.write_buffer(addr, &bytes)?;
    let mut temp = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            1,
            old_protect,
            &mut temp,
        )
        .map_err(|e| format!("VirtualProtectEx restore failed: {}", e))?;
        let _ = FlushInstructionCache(process.handle, Some(addr as *const c_void), 1);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn patch_remote_byte(process: &ProcessHandle, addr: usize, byte_val: u8) -> Result<(), String> {
    process.write_buffer(addr, &[byte_val])
}

pub struct CombatTweaks {
    pub is_patched: bool,
    pub enabled: bool,
}

impl CombatTweaks {
    pub fn new() -> Self {
        Self {
            is_patched: false,
            enabled: true,
        }
    }

    /// Apply the continuous attack patch (changes JNE 0x75 to JMP 0xEB at D2Client + 0x5948B)
    pub fn apply(&mut self, process: &ProcessHandle, d2_client: usize) -> Result<(), String> {
        if d2_client == 0 {
            return Err("D2Client.dll base is 0".to_string());
        }
        let target = d2_client + CONTINUOUS_ATTACK_PATCH;
        let cur = process.read_memory::<u8>(target)?;
        if cur == CONTINUOUS_ATTACK_PATCHED {
            self.is_patched = true;
            return Ok(());
        }
        if cur != CONTINUOUS_ATTACK_ORIGINAL {
            return Err(format!(
                "Continuous attack patch mismatch at {:#x}: expected {:#x}, got {:#x}",
                target, CONTINUOUS_ATTACK_ORIGINAL, cur
            ));
        }

        patch_remote_byte(process, target, CONTINUOUS_ATTACK_PATCHED)?;
        self.is_patched = true;
        crate::logger::info("CombatTweaks: Continuous attack patch applied (JMP 0xEB)");
        Ok(())
    }

    /// Restore the original instruction (JNE 0x75 at D2Client + 0x5948B)
    pub fn restore(&mut self, process: &ProcessHandle, d2_client: usize) -> Result<(), String> {
        if d2_client == 0 || !self.is_patched {
            return Ok(());
        }
        let target = d2_client + CONTINUOUS_ATTACK_PATCH;
        let cur = process.read_memory::<u8>(target)?;
        if cur == CONTINUOUS_ATTACK_PATCHED {
            patch_remote_byte(process, target, CONTINUOUS_ATTACK_ORIGINAL)?;
            crate::logger::info("CombatTweaks: Continuous attack patch restored (JNE 0x75)");
        }
        self.is_patched = false;
        Ok(())
    }

    pub fn set_enabled(
        &mut self,
        process: &ProcessHandle,
        d2_client: usize,
        enabled: bool,
    ) -> Result<(), String> {
        self.enabled = enabled;
        if enabled {
            self.apply(process, d2_client)
        } else {
            self.restore(process, d2_client)
        }
    }
}

pub struct CombatTweaksState {
    pub tweaks: Mutex<CombatTweaks>,
}

impl CombatTweaksState {
    pub fn new() -> Self {
        Self {
            tweaks: Mutex::new(CombatTweaks::new()),
        }
    }
}

/// Tauri command to toggle continuous attack
#[tauri::command]
pub fn toggle_continuous_attack(
    state: tauri::State<'_, CombatTweaksState>,
    app_state: tauri::State<'_, crate::AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut tweaks = state.tweaks.lock().map_err(|e| e.to_string())?;
    tweaks.enabled = enabled;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    if let Ok(guard) = app_state.scanner_shared_state.read() {
        if let Some(shared) = guard.as_ref() {
            tweaks.set_enabled(&shared.ctx.process, shared.ctx.d2_client, enabled)?;
        }
    }
    Ok(())
}
