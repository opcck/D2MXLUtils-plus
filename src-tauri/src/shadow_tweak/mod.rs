//! Shadow Tweak Module
//! Disables unit and scene shadow rendering by patching D2Client MISC_CalculateShadowRGBA

use crate::offsets::d2client::{
    REMOVE_SHADOW_ORIGINAL, REMOVE_SHADOW_PATCH, REMOVE_SHADOW_PATCHED,
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
fn patch_remote_bytes(process: &ProcessHandle, addr: usize, bytes: &[u8]) -> Result<(), String> {
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            bytes.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
        .map_err(|e| format!("VirtualProtectEx RWX failed: {}", e))?;
    }
    process.write_buffer(addr, bytes)?;
    let mut temp = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            bytes.len(),
            old_protect,
            &mut temp,
        )
        .map_err(|e| format!("VirtualProtectEx restore failed: {}", e))?;
        let _ = FlushInstructionCache(process.handle, Some(addr as *const c_void), bytes.len());
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn patch_remote_bytes(process: &ProcessHandle, addr: usize, bytes: &[u8]) -> Result<(), String> {
    process.write_buffer(addr, bytes)
}

pub struct ShadowTweak {
    pub is_patched: bool,
    pub enabled: bool,
}

impl ShadowTweak {
    pub fn new() -> Self {
        Self {
            is_patched: false,
            enabled: false,
        }
    }

    /// Apply the remove shadow patch (replaces 31 bytes at D2Client + 0xA9A20)
    pub fn apply(&mut self, process: &ProcessHandle, d2_client: usize) -> Result<(), String> {
        if d2_client == 0 {
            return Err("D2Client.dll base is 0".to_string());
        }
        let target = d2_client + REMOVE_SHADOW_PATCH;
        let cur = process.read_memory::<[u8; 31]>(target)?;
        if cur == REMOVE_SHADOW_PATCHED {
            self.is_patched = true;
            return Ok(());
        }
        if cur != REMOVE_SHADOW_ORIGINAL {
            return Err(format!(
                "Remove shadow patch mismatch at {:#x}: unexpected initial bytes",
                target
            ));
        }

        patch_remote_bytes(process, target, &REMOVE_SHADOW_PATCHED)?;
        self.is_patched = true;
        crate::logger::info("ShadowTweak: Remove shadow patch applied (31 bytes)");
        Ok(())
    }

    /// Restore the original instruction (31 bytes at D2Client + 0xA9A20)
    pub fn restore(&mut self, process: &ProcessHandle, d2_client: usize) -> Result<(), String> {
        if d2_client == 0 || !self.is_patched {
            return Ok(());
        }
        let target = d2_client + REMOVE_SHADOW_PATCH;
        let cur = process.read_memory::<[u8; 31]>(target)?;
        if cur == REMOVE_SHADOW_PATCHED {
            patch_remote_bytes(process, target, &REMOVE_SHADOW_ORIGINAL)?;
            crate::logger::info("ShadowTweak: Remove shadow patch restored");
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

pub struct ShadowTweakState {
    pub tweak: Mutex<ShadowTweak>,
}

impl ShadowTweakState {
    pub fn new() -> Self {
        Self {
            tweak: Mutex::new(ShadowTweak::new()),
        }
    }
}

/// Tauri command to toggle remove shadow tweak
#[tauri::command]
pub fn toggle_remove_shadows(
    state: tauri::State<'_, ShadowTweakState>,
    app_state: tauri::State<'_, crate::AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut tweak = state.tweak.lock().map_err(|e| e.to_string())?;
    tweak.enabled = enabled;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    if let Ok(guard) = app_state.scanner_shared_state.read() {
        if let Some(shared) = guard.as_ref() {
            tweak.set_enabled(&shared.ctx.process, shared.ctx.d2_client, enabled)?;
        }
    }
    crate::logger::info(&format!("ShadowTweak toggled: enabled = {}", enabled));
    Ok(())
}
