//! Native Monster Lifebar Info Hook for D2Sigma.dll
//! Displays monster name, Class ID and 6-element resistances (Phys, Magic, Fire, Light, Cold, Poison)
//! directly in the game's top monster and boss lifebars using native Diablo II color codes (ÿc).

use crate::offsets::d2common;
use crate::offsets::d2sigma::{
    SigmaVersionOffsets, SIGMA_OFFSETS_2_10, SIGMA_OFFSETS_2_10_3, SIGMA_OFFSETS_2_14_2,
    SIGMA_OFFSETS_2_14_3, SIGMA_OFFSETS_2_9_2,
};
use crate::process::ProcessHandle;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::FlushInstructionCache;
#[cfg(target_os = "windows")]
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualProtectEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    PAGE_PROTECTION_FLAGS,
};

/// Trampoline data offsets
const DATA_OFFSET: usize = 0x300;
const TRAMPOLINE_ALLOC_SIZE: usize = 0x1000; // 4KB

// Offsets inside data segment
const OFFSET_ENABLED: usize = 0x00;
const OFFSET_SHOW_ID: usize = 0x04;
const OFFSET_ORIG_GET_NAME: usize = 0x08;
const OFFSET_D2COMMON_GET_STAT: usize = 0x0C;
const OFFSET_FORMATTED_BUF: usize = 0x80; // 256 wchar_t buffer (512 bytes)

pub struct MonsterInfoHook {
    monster_lifebar_addr: usize,
    boss_lifebar_addr: usize,
    check_display_addr: usize,
    trampoline_addr: usize,
    is_injected: bool,
    pub enabled: bool,
    pub show_id: bool,
}

impl MonsterInfoHook {
    pub fn new() -> Self {
        Self {
            monster_lifebar_addr: 0,
            boss_lifebar_addr: 0,
            check_display_addr: 0,
            trampoline_addr: 0,
            is_injected: false,
            enabled: true,
            show_id: true,
        }
    }

    pub fn reset_injection_state(&mut self) {
        self.is_injected = false;
        self.trampoline_addr = 0;
        self.monster_lifebar_addr = 0;
        self.boss_lifebar_addr = 0;
        self.check_display_addr = 0;
    }

    pub fn is_injected(&self) -> bool {
        self.is_injected
    }

    /// Resolves target RVA offsets for D2Sigma using dual-track detection:
    /// Track 1: Known PE TimeDateStamp lookup (2.14.3, 2.14.2, 2.10.x, etc.)
    /// Track 2: Upward-compatible AOB pattern scan for unknown future patches.
    fn resolve_offsets(
        process: &ProcessHandle,
        d2_sigma_base: usize,
    ) -> Result<SigmaVersionOffsets, String> {
        let e_lfanew: u32 = process
            .read_memory(d2_sigma_base + 0x3C)
            .map_err(|e| format!("read D2Sigma e_lfanew: {}", e))?;
        let timestamp: u32 = process
            .read_memory(d2_sigma_base + e_lfanew as usize + 4 + 4)
            .map_err(|e| format!("read D2Sigma TimeDateStamp: {}", e))?;

        match timestamp {
            0x6AABB2F7 => {
                crate::logger::info("MonsterInfoHook: Matched Median XL 2.14.3 static table");
                return Ok(SIGMA_OFFSETS_2_14_3);
            }
            0x6AA069AD => {
                crate::logger::info("MonsterInfoHook: Matched Median XL 2.14.2 static table");
                return Ok(SIGMA_OFFSETS_2_14_2);
            }
            0x673ECCE8 => {
                crate::logger::info("MonsterInfoHook: Matched Median XL 2.10.3 static table");
                return Ok(SIGMA_OFFSETS_2_10_3);
            }
            0x6724FDBD => {
                crate::logger::info("MonsterInfoHook: Matched Median XL 2.10 static table");
                return Ok(SIGMA_OFFSETS_2_10);
            }
            0x663D01B3 => {
                crate::logger::info("MonsterInfoHook: Matched Median XL 2.9.2 static table");
                return Ok(SIGMA_OFFSETS_2_9_2);
            }
            _ => {
                crate::logger::info(&format!(
                    "MonsterInfoHook: Unknown D2Sigma TimeDateStamp 0x{:08X} — engaging AOB pattern scan (Track 2)",
                    timestamp
                ));
            }
        }

        // Track 2: AOB Pattern Scan across first 1.5MB of D2Sigma .text section
        let scan_size = 0x180000;
        let d2sigma_bytes = process
            .read_buffer(d2_sigma_base, scan_size)
            .map_err(|e| format!("read D2Sigma for AOB scan: {}", e))?;

        // 1. Scan for MonsterLifeBar: 8B CD E8 ?? ?? ?? ?? 8D 54 24
        let mut monster_rva = None;
        let mut units_get_name_rva = None;
        for i in 0..(d2sigma_bytes.len().saturating_sub(12)) {
            if d2sigma_bytes[i] == 0x8B
                && d2sigma_bytes[i + 1] == 0xCD
                && d2sigma_bytes[i + 2] == 0xE8
                && d2sigma_bytes[i + 7] == 0x8D
                && d2sigma_bytes[i + 8] == 0x54
                && d2sigma_bytes[i + 9] == 0x24
            {
                let call_rva = i + 2;
                let rel = i32::from_le_bytes([
                    d2sigma_bytes[i + 3],
                    d2sigma_bytes[i + 4],
                    d2sigma_bytes[i + 5],
                    d2sigma_bytes[i + 6],
                ]);
                let target = (call_rva + 5) as isize + rel as isize;
                monster_rva = Some(call_rva);
                units_get_name_rva = Some(target as usize);
                break;
            }
        }

        // 2. Scan for BossLifeBar: 8B CB E8 ?? ?? ?? ?? 8B D0 C7 45
        let mut boss_rva = None;
        for i in 0..(d2sigma_bytes.len().saturating_sub(14)) {
            if d2sigma_bytes[i] == 0x8B
                && d2sigma_bytes[i + 1] == 0xCB
                && d2sigma_bytes[i + 2] == 0xE8
                && d2sigma_bytes[i + 7] == 0x8B
                && d2sigma_bytes[i + 8] == 0xD0
                && d2sigma_bytes[i + 9] == 0xC7
                && d2sigma_bytes[i + 10] == 0x45
            {
                boss_rva = Some(i + 2);
                break;
            }
        }

        // 3. Scan for CheckIsDisplay: A9 01 02 00 00 0F 85
        let mut check_display_rva = None;
        for i in 0..(d2sigma_bytes.len().saturating_sub(8)) {
            if d2sigma_bytes[i] == 0xA9
                && d2sigma_bytes[i + 1] == 0x01
                && d2sigma_bytes[i + 2] == 0x02
                && d2sigma_bytes[i + 3] == 0x00
                && d2sigma_bytes[i + 4] == 0x00
                && d2sigma_bytes[i + 5] == 0x0F
                && d2sigma_bytes[i + 6] == 0x85
            {
                check_display_rva = Some(i + 1); // target is the '01' byte
                break;
            }
        }

        // 4. Scan for GetItemName: 81 EC 88 02 00 00 53 55 56 8B B4 24 98 02 00 00 57 85 F6
        let mut get_item_name_rva = None;
        let get_item_name_pattern: [u8; 19] = [
            0x81, 0xEC, 0x88, 0x02, 0x00, 0x00, 0x53, 0x55, 0x56, 0x8B, 0xB4, 0x24, 0x98, 0x02,
            0x00, 0x00, 0x57, 0x85, 0xF6,
        ];
        for i in 0..(d2sigma_bytes
            .len()
            .saturating_sub(get_item_name_pattern.len()))
        {
            if d2sigma_bytes[i..i + get_item_name_pattern.len()] == get_item_name_pattern {
                get_item_name_rva = Some(i);
                break;
            }
        }

        if let (Some(m), Some(b), Some(c), Some(u), Some(g)) = (
            monster_rva,
            boss_rva,
            check_display_rva,
            units_get_name_rva,
            get_item_name_rva,
        ) {
            crate::logger::info(&format!(
                "MonsterInfoHook: AOB Auto-Resolved! monster_lifebar=0x{:X}, boss_lifebar=0x{:X}, check_display=0x{:X}, units_get_name=0x{:X}, get_item_name=0x{:X}",
                m, b, c, u, g
            ));
            Ok(SigmaVersionOffsets {
                timestamp,
                get_item_name: g,
                boss_lifebar_call_units_get_name: b,
                monster_lifebar_call_units_get_name: m,
                check_is_monster_should_display_lifebar: c,
                units_get_name: u,
            })
        } else {
            crate::logger::info(
                "MonsterInfoHook: AOB scan incomplete, falling back to 2.14.3 defaults",
            );
            Ok(SIGMA_OFFSETS_2_14_3)
        }
    }

    /// Injects the Monster Info Hook into D2Sigma.dll
    pub fn inject(
        &mut self,
        process: &ProcessHandle,
        d2_sigma_base: usize,
        d2_common_base: usize,
    ) -> Result<(), String> {
        if self.is_injected {
            return Ok(());
        }
        if d2_sigma_base == 0 || d2_common_base == 0 {
            return Err("D2Sigma.dll or D2Common.dll base is 0".to_string());
        }

        let offsets = Self::resolve_offsets(process, d2_sigma_base)?;
        self.monster_lifebar_addr = d2_sigma_base + offsets.monster_lifebar_call_units_get_name;
        self.boss_lifebar_addr = d2_sigma_base + offsets.boss_lifebar_call_units_get_name;
        self.check_display_addr = d2_sigma_base + offsets.check_is_monster_should_display_lifebar;

        let orig_units_get_name_addr = (d2_sigma_base + offsets.units_get_name) as u32;
        let d2common_get_stat_addr = (d2_common_base + d2common::GET_UNIT_STAT) as u32;

        // Verify call sites (first byte must be 0xE8 call)
        let m_first: u8 = process
            .read_memory(self.monster_lifebar_addr)
            .map_err(|e| format!("verify monster lifebar hook site: {}", e))?;
        if m_first != 0xE8 {
            return Err(format!(
                "MonsterLifeBar hook site mismatch at 0x{:X}: expected 0xE8, got 0x{:02X}",
                self.monster_lifebar_addr, m_first
            ));
        }

        // Allocate memory for trampoline + data
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
            return Err("VirtualAllocEx failed for MonsterInfo trampoline".to_string());
        }
        self.trampoline_addr = trampoline_addr;

        let g_enabled = (trampoline_addr + DATA_OFFSET + OFFSET_ENABLED) as u32;
        let g_show_id = (trampoline_addr + DATA_OFFSET + OFFSET_SHOW_ID) as u32;
        let g_orig_get_name = (trampoline_addr + DATA_OFFSET + OFFSET_ORIG_GET_NAME) as u32;
        let g_d2common_get_stat = (trampoline_addr + DATA_OFFSET + OFFSET_D2COMMON_GET_STAT) as u32;
        let g_formatted_buf = (trampoline_addr + DATA_OFFSET + OFFSET_FORMATTED_BUF) as u32;

        // Build trampoline bytecode
        let mut code = Vec::with_capacity(512);

        // --- Entry point ---
        // mov eax, [g_enabled]
        code.push(0xA1);
        code.extend_from_slice(&g_enabled.to_le_bytes());
        // test eax, eax
        code.extend_from_slice(&[0x85, 0xC0]);
        // jnz .do_format
        let do_format_jmp_idx = code.len();
        code.extend_from_slice(&[0x0F, 0x85, 0, 0, 0, 0]);

        // .fallback:
        // jmp dword ptr [g_orig_get_name] (FF 25 [addr])
        code.extend_from_slice(&[0xFF, 0x25]);
        code.extend_from_slice(&g_orig_get_name.to_le_bytes());

        // .do_format:
        let do_format_offset = code.len();
        let rel_do_format = (do_format_offset as isize - (do_format_jmp_idx as isize + 6)) as i32;
        code[do_format_jmp_idx + 2..do_format_jmp_idx + 6]
            .copy_from_slice(&rel_do_format.to_le_bytes());

        // push ebp; mov ebp, esp; push ebx; push esi; push edi
        code.extend_from_slice(&[0x55, 0x89, 0xE5, 0x53, 0x56, 0x57]);
        // mov esi, ecx  (save pUnit in ESI)
        code.extend_from_slice(&[0x89, 0xCE]);

        // ensure ecx = esi before calling Units_GetName (fastcall)
        code.extend_from_slice(&[0x89, 0xF1]);
        // call dword ptr [g_orig_get_name]
        code.extend_from_slice(&[0xFF, 0x15]);
        code.extend_from_slice(&g_orig_get_name.to_le_bytes());
        // mov ebx, eax  (EBX = pOrigName wchar_t*)
        code.extend_from_slice(&[0x89, 0xC3]);

        // mov edi, g_formatted_buf
        code.push(0xBF);
        code.extend_from_slice(&g_formatted_buf.to_le_bytes());

        // Check if pOrigName is NULL
        code.extend_from_slice(&[0x85, 0xDB]);
        let null_name_jmp_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz .name_done

        // .copy_orig_name (limit to max 12 characters to prevent stack overflow in D2Sigma caller)
        // xor edx, edx
        code.extend_from_slice(&[0x31, 0xD2]);
        let copy_loop_offset = code.len();
        code.extend_from_slice(&[0x83, 0xFA, 0x0C]); // cmp edx, 12
        let max_name_jmp_idx = code.len();
        code.extend_from_slice(&[0x7D, 0]); // short jge .name_done
        code.extend_from_slice(&[0x66, 0x8B, 0x0B, 0x66, 0x85, 0xC9]);
        let name_done_jmp_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz .name_done
        code.extend_from_slice(&[
            0x66, 0x89, 0x0F, 0x83, 0xC3, 0x02, 0x83, 0xC7, 0x02, 0x42, 0xEB,
        ]);
        let loop_back = (copy_loop_offset as isize - (code.len() as isize + 1)) as i8;
        code.push(loop_back as u8);

        // .name_done:
        let name_done_offset = code.len();
        code[null_name_jmp_idx + 1] = (name_done_offset - (null_name_jmp_idx + 2)) as u8;
        code[max_name_jmp_idx + 1] = (name_done_offset - (max_name_jmp_idx + 2)) as u8;
        code[name_done_jmp_idx + 1] = (name_done_offset - (name_done_jmp_idx + 2)) as u8;

        // Check show_id
        // mov eax, [g_show_id]
        code.push(0xA1);
        code.extend_from_slice(&g_show_id.to_le_bytes());
        // test eax, eax; jz .skip_id
        code.extend_from_slice(&[0x85, 0xC0]);
        let skip_id_jmp_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz

        // append '[' -> 0x005B
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x5B, 0x00, 0x83, 0xC7, 0x02]);

        // eax = [esi + 4] (dwClassId)
        code.extend_from_slice(&[0x8B, 0x46, 0x04]);
        // call append_dec
        let call_append_dec_idx1 = code.len();
        code.extend_from_slice(&[0xE8, 0, 0, 0, 0]);

        // append ']' -> 0x005D
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x5D, 0x00, 0x83, 0xC7, 0x02]);

        // .skip_id:
        let skip_id_offset = code.len();
        code[skip_id_jmp_idx + 1] = (skip_id_offset - (skip_id_jmp_idx + 2)) as u8;

        // append space ' ' -> 0x0020
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x20, 0x00, 0x83, 0xC7, 0x02]);

        // Helper macro/closure to append resistance stat with color prefix
        let mut append_stat_call_site_indices = Vec::new();
        let mut emit_stat_block =
            |code: &mut Vec<u8>,
             stat_id: u32,
             color_code_char: u8,
             call_indices: &mut Vec<usize>| {
                // append ÿc{color} -> 0x00FF, 0x0063, 0x00{color}
                code.extend_from_slice(&[
                    0x66,
                    0xC7,
                    0x07,
                    0xFF,
                    0x00,
                    0x66,
                    0xC7,
                    0x47,
                    0x02,
                    0x63,
                    0x00,
                    0x66,
                    0xC7,
                    0x47,
                    0x04,
                    color_code_char,
                    0x00,
                    0x83,
                    0xC7,
                    0x06,
                ]);
                // push 0; push stat_id; push esi; call dword ptr [g_d2common_get_stat]
                code.extend_from_slice(&[0x6A, 0x00, 0x68]);
                code.extend_from_slice(&stat_id.to_le_bytes());
                code.extend_from_slice(&[0x56, 0xFF, 0x15]);
                code.extend_from_slice(&g_d2common_get_stat.to_le_bytes());
                // call append_stat
                let call_idx = code.len();
                code.extend_from_slice(&[0xE8, 0, 0, 0, 0]);
                call_indices.push(call_idx);
                // append space ' '
                code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x20, 0x00, 0x83, 0xC7, 0x02]);
            };

        // 1. Phys: stat 36, color '7' (0x37)
        emit_stat_block(&mut code, 36, 0x37, &mut append_stat_call_site_indices);
        // 2. Magic: stat 37, color '8' (0x38)
        emit_stat_block(&mut code, 37, 0x38, &mut append_stat_call_site_indices);
        // 3. Fire: stat 39, color '1' (0x31)
        emit_stat_block(&mut code, 39, 0x31, &mut append_stat_call_site_indices);
        // 4. Light: stat 41, color '9' (0x39)
        emit_stat_block(&mut code, 41, 0x39, &mut append_stat_call_site_indices);
        // 5. Cold: stat 43, color '3' (0x33)
        emit_stat_block(&mut code, 43, 0x33, &mut append_stat_call_site_indices);
        // 6. Poison: stat 45, color '2' (0x32)
        emit_stat_block(&mut code, 45, 0x32, &mut append_stat_call_site_indices);

        // Remove trailing space by decrementing edi by 2
        code.extend_from_slice(&[0x83, 0xEF, 0x02]);

        // Null terminator: mov word ptr [edi], 0x0000
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x00, 0x00]);

        // Return formatted buffer address in EAX
        code.push(0xB8);
        code.extend_from_slice(&g_formatted_buf.to_le_bytes());

        // Restore registers & return
        code.extend_from_slice(&[0x5F, 0x5E, 0x5B, 0x89, 0xEC, 0x5D, 0xC3]);

        // --- SUBROUTINES ---

        // Subroutine: append_stat (handles negative signed values, then delegates to append_dec)
        let sub_append_stat_offset = code.len();
        // test eax, eax
        // jns .stat_positive
        // mov word ptr [edi], 0x002D ('-'); add edi, 2; neg eax
        // .stat_positive:
        // jmp append_dec
        code.extend_from_slice(&[
            0x85, 0xC0, 0x79, 0x0B, 0x66, 0xC7, 0x07, 0x2D, 0x00, 0x83, 0xC7, 0x02, 0xF7, 0xD8,
        ]);
        // Fall through into append_dec!

        // Subroutine: append_dec (EAX = u32, writes decimal to EDI)
        let sub_append_dec_offset = code.len();
        code.extend_from_slice(&[
            0x53, 0x56, 0xBB, 0x0A, 0x00, 0x00, 0x00, 0x31,
            0xC9, // push ebx, esi; mov ebx, 10; xor ecx, ecx
            0x31, 0xD2, 0xF7, 0xF3, 0x52, 0x41, 0x85, 0xC0, 0x75,
            0xF6, // .div_loop: xor edx, edx; div ebx; push edx; inc ecx; test eax, eax; jnz .div_loop
            0x5A, 0x83, 0xC2, 0x30, 0x66, 0x89, 0x17, 0x83, 0xC7, 0x02, 0x49, 0x75,
            0xF2, // .pop_loop: pop edx; add edx, '0'; mov [edi], dx; add edi, 2; dec ecx; jnz .pop_loop
            0x5E, 0x5B, 0xC3, // pop esi, ebx, ret
        ]);

        // Subroutine: append_hex (EAX = u32, writes hexadecimal to EDI)
        let sub_append_hex_offset = code.len();
        code.extend_from_slice(&[
            0x53, 0x56, 0xBB, 0x10, 0x00, 0x00, 0x00, 0x31,
            0xC9, // push ebx, esi; mov ebx, 16; xor ecx, ecx
            0x31, 0xD2, 0xF7, 0xF3, 0x52, 0x41, 0x85, 0xC0, 0x75,
            0xF6, // .hex_div: xor edx, edx; div ebx; push edx; inc ecx; test eax, eax; jnz .hex_div
            0x5A, 0x83, 0xFA, 0x0A, 0x7C, 0x05, 0x83, 0xC2, 0x37, 0xEB, 0x03, 0x83, 0xC2,
            0x30, // .hex_pop: pop edx; cmp edx, 10; jl '0'; add edx, 0x37 ('A'-10); jmp store; add edx, '0'
            0x66, 0x89, 0x17, 0x83, 0xC7, 0x02, 0x49, 0x75,
            0xE9, // mov [edi], dx; add edi, 2; dec ecx; jnz .hex_pop
            0x5E, 0x5B, 0xC3, // pop esi, ebx, ret
        ]);

        // Fix up internal CALL rel32 targets
        let fixup_call = |code: &mut Vec<u8>, call_site_idx: usize, target_offset: usize| {
            let next_ip = call_site_idx + 5;
            let rel = (target_offset as isize - next_ip as isize) as i32;
            code[call_site_idx + 1..call_site_idx + 5].copy_from_slice(&rel.to_le_bytes());
        };

        fixup_call(&mut code, call_append_dec_idx1, sub_append_dec_offset);
        for call_idx in append_stat_call_site_indices {
            fixup_call(&mut code, call_idx, sub_append_stat_offset);
        }

        // 3. Write trampoline code to remote memory
        process.write_buffer(trampoline_addr, &code)?;

        // 4. Initialize trampoline data
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_ENABLED,
            &(self.enabled as u32).to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_SHOW_ID,
            &(self.show_id as u32).to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_ORIG_GET_NAME,
            &orig_units_get_name_addr.to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_D2COMMON_GET_STAT,
            &d2common_get_stat_addr.to_le_bytes(),
        )?;

        // 5. Patch CheckIsDisplay: byte at check_display_addr modified from 0x01 to 0x80
        let cur_disp_byte: u8 = process.read_memory(self.check_display_addr)?;
        if cur_disp_byte == 0x01 {
            patch_remote_byte(process, self.check_display_addr, 0x80)?;
            crate::logger::info("MonsterInfoHook: Patched CheckIsDisplay (0x01 -> 0x80)");
        }

        // 6. Patch call sites to jump to our trampoline entry point
        // MonsterLifeBar: E8 rel32
        patch_call_site(process, self.monster_lifebar_addr, trampoline_addr)?;
        // BossLifeBar: E8 rel32
        patch_call_site(process, self.boss_lifebar_addr, trampoline_addr)?;

        self.is_injected = true;
        crate::logger::info(&format!(
            "MonsterInfoHook: Injected successfully! trampoline=0x{:08X}, monster_call=0x{:08X}, boss_call=0x{:08X}",
            trampoline_addr, self.monster_lifebar_addr, self.boss_lifebar_addr
        ));

        Ok(())
    }

    /// Dynamically update enabled flag in remote memory (instant in-game switch)
    pub fn set_enabled(&mut self, process: &ProcessHandle, enabled: bool) -> Result<(), String> {
        self.enabled = enabled;
        if self.is_injected && self.trampoline_addr != 0 {
            process.write_buffer(
                self.trampoline_addr + DATA_OFFSET + OFFSET_ENABLED,
                &(enabled as u32).to_le_bytes(),
            )?;
        }
        Ok(())
    }

    /// Dynamically update show_id flag in remote memory
    pub fn set_show_id(&mut self, process: &ProcessHandle, show_id: bool) -> Result<(), String> {
        self.show_id = show_id;
        if self.is_injected && self.trampoline_addr != 0 {
            process.write_buffer(
                self.trampoline_addr + DATA_OFFSET + OFFSET_SHOW_ID,
                &(show_id as u32).to_le_bytes(),
            )?;
        }
        Ok(())
    }
}

/// Helper to patch a 5-byte E8 rel32 call instruction
fn patch_call_site(
    process: &ProcessHandle,
    call_addr: usize,
    target_addr: usize,
) -> Result<(), String> {
    let rel = (target_addr as isize - (call_addr as isize + 5)) as i32;
    let mut bytes = [0u8; 5];
    bytes[0] = 0xE8;
    bytes[1..5].copy_from_slice(&rel.to_le_bytes());

    #[cfg(target_os = "windows")]
    unsafe {
        let mut old_protect = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            process.handle,
            call_addr as *const c_void,
            5,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
        .map_err(|e| format!("VirtualProtectEx RWX failed at 0x{:X}: {}", call_addr, e))?;

        process.write_buffer(call_addr, &bytes)?;

        let mut temp = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            process.handle,
            call_addr as *const c_void,
            5,
            old_protect,
            &mut temp,
        )
        .map_err(|e| {
            format!(
                "VirtualProtectEx restore failed at 0x{:X}: {}",
                call_addr, e
            )
        })?;

        let _ = FlushInstructionCache(process.handle, Some(call_addr as *const c_void), 5);
    }
    #[cfg(not(target_os = "windows"))]
    process.write_buffer(call_addr, &bytes)?;

    Ok(())
}

/// Helper to patch a single remote byte safely
fn patch_remote_byte(process: &ProcessHandle, addr: usize, byte_val: u8) -> Result<(), String> {
    let bytes = [byte_val];
    #[cfg(target_os = "windows")]
    unsafe {
        let mut old_protect = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            1,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
        .map_err(|e| format!("VirtualProtectEx RWX failed at 0x{:X}: {}", addr, e))?;

        process.write_buffer(addr, &bytes)?;

        let mut temp = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            process.handle,
            addr as *const c_void,
            1,
            old_protect,
            &mut temp,
        )
        .map_err(|e| format!("VirtualProtectEx restore failed at 0x{:X}: {}", addr, e))?;

        let _ = FlushInstructionCache(process.handle, Some(addr as *const c_void), 1);
    }
    #[cfg(not(target_os = "windows"))]
    process.write_buffer(addr, &bytes)?;

    Ok(())
}
