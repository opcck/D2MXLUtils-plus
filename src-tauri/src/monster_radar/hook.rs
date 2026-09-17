//! Native automap monster blob drawing hook for D2Sigma.dll
//! Injects a high-performance inline trampoline into the native automap
//! loop (RVA 0x485D3) to display Normal, Elite, and Boss monsters on the minimap.

use super::boss_data::generate_boss_lookup_table;
use crate::offsets::d2sigma::{
    DRAW_AUTOMAP_BLOB_FN, DRAW_MONSTER_BLOB_EXIT, DRAW_MONSTER_BLOB_HOOK,
    DRAW_MONSTER_BLOB_ORIGINAL_BYTES, DRAW_MONSTER_BLOB_PATCH_SIZE, DRAW_MONSTER_BLOB_RESUME,
};
use crate::process::ProcessHandle;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::FlushInstructionCache;
#[cfg(target_os = "windows")]
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, VirtualProtectEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

/// Data section offset inside our 8192-byte allocated trampoline page.
const DATA_OFFSET: usize = 0x200;
const TRAMPOLINE_ALLOC_SIZE: usize = 0x2000; // 8KB (plenty for code + 4KB lookup table)

/// Default radar colors (D2 palette indices)
pub const DEFAULT_COLOR_NORMAL: u32 = 0x62; // Red dot
pub const DEFAULT_COLOR_BOSS: u32 = 0x5B; // Golden dot
pub const DEFAULT_COLOR_ELITE: u32 = 0x9A; // Purple dot

pub struct MonsterRadarHook {
    hook_address: usize,
    trampoline_address: usize,
    is_injected: bool,
    pub enabled: bool,
    pub show_normal: bool,
}

impl MonsterRadarHook {
    pub fn new() -> Self {
        Self {
            hook_address: 0,
            trampoline_address: 0,
            is_injected: false,
            enabled: true,
            show_normal: true,
        }
    }

    pub fn is_injected(&self) -> bool {
        self.is_injected
    }

    /// Injects the radar hook into D2Sigma.dll.
    pub fn inject(&mut self, process: &ProcessHandle, d2_sigma_base: usize) -> Result<(), String> {
        if self.is_injected {
            return Ok(());
        }
        if d2_sigma_base == 0 {
            return Err("D2Sigma.dll base is 0".to_string());
        }

        let hook_addr = d2_sigma_base + DRAW_MONSTER_BLOB_HOOK;
        let resume_addr = d2_sigma_base + DRAW_MONSTER_BLOB_RESUME;
        let exit_addr = d2_sigma_base + DRAW_MONSTER_BLOB_EXIT;
        let draw_blob_addr = d2_sigma_base + DRAW_AUTOMAP_BLOB_FN;

        // 1. Verify original bytes at hook point
        let mut cur_bytes = [0u8; DRAW_MONSTER_BLOB_PATCH_SIZE];
        process.read_buffer_into(hook_addr, &mut cur_bytes)?;
        if cur_bytes != DRAW_MONSTER_BLOB_ORIGINAL_BYTES {
            // Already hooked or modified
            if cur_bytes[0] == 0xE9 {
                crate::logger::info("MonsterRadarHook: Hook point already patched (0xE9 JMP)");
                self.is_injected = true;
                return Ok(());
            }
            return Err(format!(
                "MonsterRadarHook: byte mismatch at {:#x}: expected {:02x?}, found {:02x?}",
                hook_addr, DRAW_MONSTER_BLOB_ORIGINAL_BYTES, cur_bytes
            ));
        }

        // 2. Allocate memory for trampoline + data
        #[cfg(target_os = "windows")]
        let trampoline_addr = unsafe {
            VirtualAllocEx(
                process.handle,
                None,
                TRAMPOLINE_ALLOC_SIZE,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            ) as usize
        };
        #[cfg(not(target_os = "windows"))]
        let trampoline_addr = 0usize;

        if trampoline_addr == 0 {
            return Err("VirtualAllocEx failed for MonsterRadar trampoline".to_string());
        }

        // Data offsets relative to trampoline_addr
        let g_radar_enabled = (trampoline_addr + DATA_OFFSET) as u32;
        let g_show_normal = (trampoline_addr + DATA_OFFSET + 0x04) as u32;
        let g_color_normal = (trampoline_addr + DATA_OFFSET + 0x08) as u32;
        let g_color_boss = (trampoline_addr + DATA_OFFSET + 0x0C) as u32;
        let g_color_elite = (trampoline_addr + DATA_OFFSET + 0x10) as u32;
        let g_boss_table = (trampoline_addr + DATA_OFFSET + 0x20) as u32;

        // 3. Assemble trampoline bytecode
        let mut code = Vec::with_capacity(256);

        // Helper closures
        let emit_call_rel32 = |code: &mut Vec<u8>, target: usize| {
            let cur = trampoline_addr + code.len() + 5;
            let rel = (target as isize - cur as isize) as i32;
            code.push(0xE8);
            code.extend_from_slice(&rel.to_le_bytes());
        };
        let emit_jmp_rel32 = |code: &mut Vec<u8>, target: usize| {
            let cur = trampoline_addr + code.len() + 5;
            let rel = (target as isize - cur as isize) as i32;
            code.push(0xE9);
            code.extend_from_slice(&rel.to_le_bytes());
        };

        // pushfd; push eax; push ecx; push edx; push ebx
        code.extend_from_slice(&[0x9C, 0x50, 0x51, 0x52, 0x53]);

        // mov eax, [g_radar_enabled]
        code.push(0xA1);
        code.extend_from_slice(&g_radar_enabled.to_le_bytes());
        // test eax, eax
        code.extend_from_slice(&[0x85, 0xC0]);
        // jz fallback (placeholder rel32)
        let fallback_jmp_idx = code.len();
        code.extend_from_slice(&[0x0F, 0x84, 0, 0, 0, 0]);

        // mov eax, [edi + 4] (dwClassId)
        code.extend_from_slice(&[0x8B, 0x47, 0x04]);
        // cmp eax, 4096
        code.extend_from_slice(&[0x3D, 0x00, 0x10, 0x00, 0x00]);
        // jae check_quality
        let check_quality_idx1 = code.len();
        code.extend_from_slice(&[0x0F, 0x83, 0, 0, 0, 0]);

        // movzx ecx, byte ptr [eax + g_boss_table]
        code.extend_from_slice(&[0x0F, 0xB6, 0x88]);
        code.extend_from_slice(&g_boss_table.to_le_bytes());
        // cmp ecx, 1
        code.extend_from_slice(&[0x83, 0xF9, 0x01]);
        // jne check_quality
        let check_quality_idx2 = code.len();
        code.extend_from_slice(&[0x0F, 0x85, 0, 0, 0, 0]);

        // Is Boss! mov edx, [g_color_boss]
        code.extend_from_slice(&[0x8B, 0x15]);
        code.extend_from_slice(&g_color_boss.to_le_bytes());
        // jmp draw_blob
        let draw_blob_idx1 = code.len();
        code.extend_from_slice(&[0xE9, 0, 0, 0, 0]);

        // --- check_quality label ---
        let cq_offset = code.len();
        let rel1 = (cq_offset as isize - (check_quality_idx1 as isize + 6)) as i32;
        code[check_quality_idx1 + 2..check_quality_idx1 + 6].copy_from_slice(&rel1.to_le_bytes());
        let rel2 = (cq_offset as isize - (check_quality_idx2 as isize + 6)) as i32;
        code[check_quality_idx2 + 2..check_quality_idx2 + 6].copy_from_slice(&rel2.to_le_bytes());

        // mov eax, [edi + 0x14] (pMonsterData)
        code.extend_from_slice(&[0x8B, 0x47, 0x14]);
        // test eax, eax
        code.extend_from_slice(&[0x85, 0xC0]);
        // jz check_normal
        let check_normal_idx1 = code.len();
        code.extend_from_slice(&[0x0F, 0x84, 0, 0, 0, 0]);

        // movzx ecx, byte ptr [eax + 0x16]
        code.extend_from_slice(&[0x0F, 0xB6, 0x48, 0x16]);
        // test ecx, ecx
        code.extend_from_slice(&[0x85, 0xC9]);
        // jnz is_elite
        let is_elite_idx1 = code.len();
        code.extend_from_slice(&[0x0F, 0x85, 0, 0, 0, 0]);

        // movzx ecx, byte ptr [eax + 0x18]
        code.extend_from_slice(&[0x0F, 0xB6, 0x48, 0x18]);
        // test ecx, ecx
        code.extend_from_slice(&[0x85, 0xC9]);
        // jz check_normal
        let check_normal_idx2 = code.len();
        code.extend_from_slice(&[0x0F, 0x84, 0, 0, 0, 0]);

        // --- is_elite label ---
        let ie_offset = code.len();
        let rel_ie = (ie_offset as isize - (is_elite_idx1 as isize + 6)) as i32;
        code[is_elite_idx1 + 2..is_elite_idx1 + 6].copy_from_slice(&rel_ie.to_le_bytes());

        // mov edx, [g_color_elite]
        code.extend_from_slice(&[0x8B, 0x15]);
        code.extend_from_slice(&g_color_elite.to_le_bytes());
        // jmp draw_blob
        let draw_blob_idx2 = code.len();
        code.extend_from_slice(&[0xE9, 0, 0, 0, 0]);

        // --- check_normal label ---
        let cn_offset = code.len();
        let rel_cn1 = (cn_offset as isize - (check_normal_idx1 as isize + 6)) as i32;
        code[check_normal_idx1 + 2..check_normal_idx1 + 6].copy_from_slice(&rel_cn1.to_le_bytes());
        let rel_cn2 = (cn_offset as isize - (check_normal_idx2 as isize + 6)) as i32;
        code[check_normal_idx2 + 2..check_normal_idx2 + 6].copy_from_slice(&rel_cn2.to_le_bytes());

        // mov eax, [g_show_normal]
        code.push(0xA1);
        code.extend_from_slice(&g_show_normal.to_le_bytes());
        // test eax, eax
        code.extend_from_slice(&[0x85, 0xC0]);
        // jz skip_monster
        let skip_monster_idx1 = code.len();
        code.extend_from_slice(&[0x0F, 0x84, 0, 0, 0, 0]);

        // mov edx, [g_color_normal]
        code.extend_from_slice(&[0x8B, 0x15]);
        code.extend_from_slice(&g_color_normal.to_le_bytes());

        // --- draw_blob label ---
        let db_offset = code.len();
        let rel_db1 = (db_offset as isize - (draw_blob_idx1 as isize + 5)) as i32;
        code[draw_blob_idx1 + 1..draw_blob_idx1 + 5].copy_from_slice(&rel_db1.to_le_bytes());
        let rel_db2 = (db_offset as isize - (draw_blob_idx2 as isize + 5)) as i32;
        code[draw_blob_idx2 + 1..draw_blob_idx2 + 5].copy_from_slice(&rel_db2.to_le_bytes());

        // lea ecx, [esp + 0x20] (Point screen coordinates)
        code.extend_from_slice(&[0x8D, 0x4C, 0x24, 0x20]);
        // call draw_blob_fn
        emit_call_rel32(&mut code, draw_blob_addr);

        // --- skip_monster label ---
        let sm_offset = code.len();
        let rel_sm = (sm_offset as isize - (skip_monster_idx1 as isize + 6)) as i32;
        code[skip_monster_idx1 + 2..skip_monster_idx1 + 6].copy_from_slice(&rel_sm.to_le_bytes());

        // pop ebx; pop edx; pop ecx; pop eax; popfd
        code.extend_from_slice(&[0x5B, 0x5A, 0x59, 0x58, 0x9D]);
        // jmp exit_addr
        emit_jmp_rel32(&mut code, exit_addr);

        // --- fallback label ---
        let fb_offset = code.len();
        let rel_fb = (fb_offset as isize - (fallback_jmp_idx as isize + 6)) as i32;
        code[fallback_jmp_idx + 2..fallback_jmp_idx + 6].copy_from_slice(&rel_fb.to_le_bytes());

        // pop ebx; pop edx; pop ecx; pop eax; popfd
        code.extend_from_slice(&[0x5B, 0x5A, 0x59, 0x58, 0x9D]);
        // Replay original 8 bytes: mov eax, [ebx + 0x0C]; and eax, 0x0300
        code.extend_from_slice(&DRAW_MONSTER_BLOB_ORIGINAL_BYTES);
        // jmp resume_addr
        emit_jmp_rel32(&mut code, resume_addr);

        // 4. Prepare data block
        let mut data_buf = vec![0u8; TRAMPOLINE_ALLOC_SIZE - DATA_OFFSET];
        // +0x00: g_radar_enabled
        let enabled_val: u32 = if self.enabled { 1 } else { 0 };
        data_buf[0..4].copy_from_slice(&enabled_val.to_le_bytes());
        // +0x04: g_show_normal
        let show_normal_val: u32 = if self.show_normal { 1 } else { 0 };
        data_buf[4..8].copy_from_slice(&show_normal_val.to_le_bytes());
        // +0x08: g_color_normal
        data_buf[8..12].copy_from_slice(&DEFAULT_COLOR_NORMAL.to_le_bytes());
        // +0x0C: g_color_boss
        data_buf[12..16].copy_from_slice(&DEFAULT_COLOR_BOSS.to_le_bytes());
        // +0x10: g_color_elite
        data_buf[16..20].copy_from_slice(&DEFAULT_COLOR_ELITE.to_le_bytes());
        // +0x20: boss lookup table (4096 bytes)
        let boss_table = generate_boss_lookup_table();
        data_buf[0x20..0x20 + 4096].copy_from_slice(&boss_table);

        // 5. Write trampoline code and data to process memory
        process.write_buffer(trampoline_addr, &code)?;
        process.write_buffer(trampoline_addr + DATA_OFFSET, &data_buf)?;

        // 6. Install 5-byte JMP at hook_address + 3 NOPs
        let mut patch = [0x90u8; DRAW_MONSTER_BLOB_PATCH_SIZE];
        let jmp_rel = (trampoline_addr as isize - (hook_addr as isize + 5)) as i32;
        patch[0] = 0xE9;
        patch[1..5].copy_from_slice(&jmp_rel.to_le_bytes());

        #[cfg(target_os = "windows")]
        unsafe {
            let mut old_protect = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                process.handle,
                hook_addr as *const c_void,
                DRAW_MONSTER_BLOB_PATCH_SIZE,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
            .map_err(|e| format!("VirtualProtectEx hook failed: {}", e))?;

            process.write_buffer(hook_addr, &patch)?;

            let mut dummy = PAGE_PROTECTION_FLAGS(0);
            let _ = VirtualProtectEx(
                process.handle,
                hook_addr as *const c_void,
                DRAW_MONSTER_BLOB_PATCH_SIZE,
                old_protect,
                &mut dummy,
            );

            let _ = FlushInstructionCache(
                process.handle,
                Some(hook_addr as *const c_void),
                DRAW_MONSTER_BLOB_PATCH_SIZE,
            );
        }

        self.hook_address = hook_addr;
        self.trampoline_address = trampoline_addr;
        self.is_injected = true;

        crate::logger::info(&format!(
            "MonsterRadarHook: injected at {:#x} -> trampoline {:#x}",
            hook_addr, trampoline_addr
        ));

        Ok(())
    }

    /// Ejects the hook and restores original code.
    pub fn eject(&mut self, process: &ProcessHandle) -> Result<(), String> {
        if !self.is_injected || self.hook_address == 0 {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        unsafe {
            let mut old_protect = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                process.handle,
                self.hook_address as *const c_void,
                DRAW_MONSTER_BLOB_PATCH_SIZE,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
            .map_err(|e| format!("VirtualProtectEx eject failed: {}", e))?;

            process.write_buffer(self.hook_address, &DRAW_MONSTER_BLOB_ORIGINAL_BYTES)?;

            let mut dummy = PAGE_PROTECTION_FLAGS(0);
            let _ = VirtualProtectEx(
                process.handle,
                self.hook_address as *const c_void,
                DRAW_MONSTER_BLOB_PATCH_SIZE,
                old_protect,
                &mut dummy,
            );

            let _ = FlushInstructionCache(
                process.handle,
                Some(self.hook_address as *const c_void),
                DRAW_MONSTER_BLOB_PATCH_SIZE,
            );

            if self.trampoline_address != 0 {
                let _ = VirtualFreeEx(
                    process.handle,
                    self.trampoline_address as *mut c_void,
                    0,
                    MEM_RELEASE,
                );
            }
        }

        self.is_injected = false;
        self.hook_address = 0;
        self.trampoline_address = 0;
        crate::logger::info("MonsterRadarHook: cleanly ejected");
        Ok(())
    }

    /// Updates dynamic runtime settings without rehooking.
    pub fn set_enabled(&mut self, process: &ProcessHandle, enabled: bool) -> Result<(), String> {
        self.enabled = enabled;
        if self.is_injected && self.trampoline_address != 0 {
            let val: u32 = if enabled { 1 } else { 0 };
            process.write_buffer(self.trampoline_address + DATA_OFFSET, &val.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn set_show_normal(
        &mut self,
        process: &ProcessHandle,
        show_normal: bool,
    ) -> Result<(), String> {
        self.show_normal = show_normal;
        if self.is_injected && self.trampoline_address != 0 {
            let val: u32 = if show_normal { 1 } else { 0 };
            process.write_buffer(
                self.trampoline_address + DATA_OFFSET + 0x04,
                &val.to_le_bytes(),
            )?;
        }
        Ok(())
    }
}
