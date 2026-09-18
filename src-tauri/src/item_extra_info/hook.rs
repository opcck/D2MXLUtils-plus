//! Native Item Extra Info Hook for D2Sigma.dll
//! Intercepts D2Sigma's GetItemName to prepend UID, CID and append sockets & ethereal tags
//! directly into in-game item nameplates and tooltips.

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
const OFFSET_SHOW_SOCKS_ETH: usize = 0x04;
const OFFSET_D2COMMON_GET_STAT: usize = 0x08;
const OFFSET_ORIG_RESUME_ADDR: usize = 0x0C;
const OFFSET_TEMP_BUF: usize = 0x80; // 256 wchar_t buffer (512 bytes)

pub struct ItemExtraInfoHook {
    get_item_name_addr: usize,
    trampoline_addr: usize,
    is_injected: bool,
    pub enabled: bool,
    pub show_sockets_and_eth: bool,
}

impl ItemExtraInfoHook {
    pub fn new() -> Self {
        Self {
            get_item_name_addr: 0,
            trampoline_addr: 0,
            is_injected: false,
            enabled: true,
            show_sockets_and_eth: true,
        }
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
            0x6AABB2F7 => Ok(SIGMA_OFFSETS_2_14_3),
            0x6AA069AD => Ok(SIGMA_OFFSETS_2_14_2),
            0x673ECCE8 => Ok(SIGMA_OFFSETS_2_10_3),
            0x6724FDBD => Ok(SIGMA_OFFSETS_2_10),
            0x663D01B3 => Ok(SIGMA_OFFSETS_2_9_2),
            _ => {
                let scan_size = 0x180000;
                let d2sigma_bytes = process
                    .read_buffer(d2_sigma_base, scan_size)
                    .map_err(|e| format!("read D2Sigma for AOB scan: {}", e))?;

                let get_item_name_pattern: [u8; 19] = [
                    0x81, 0xEC, 0x88, 0x02, 0x00, 0x00, 0x53, 0x55, 0x56, 0x8B, 0xB4, 0x24, 0x98,
                    0x02, 0x00, 0x00, 0x57, 0x85, 0xF6,
                ];
                for i in 0..(d2sigma_bytes
                    .len()
                    .saturating_sub(get_item_name_pattern.len()))
                {
                    if d2sigma_bytes[i..i + get_item_name_pattern.len()] == get_item_name_pattern {
                        let mut offsets = SIGMA_OFFSETS_2_14_3;
                        offsets.get_item_name = i;
                        return Ok(offsets);
                    }
                }
                Ok(SIGMA_OFFSETS_2_14_3)
            }
        }
    }

    /// Injects the Item Extra Info Hook into D2Sigma.dll
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
        self.get_item_name_addr = d2_sigma_base + offsets.get_item_name;
        let d2common_get_stat_addr = (d2_common_base + d2common::GET_UNIT_STAT) as u32;
        let orig_resume_addr = (self.get_item_name_addr + 6) as u32;

        // Verify prologue: 81 EC 88 02 00 00 (sub esp, 288h)
        let mut prologue = [0u8; 6];
        process.read_buffer_into(self.get_item_name_addr, &mut prologue)?;
        if prologue[0] == 0xE9 {
            crate::logger::info("ItemExtraInfoHook: Already patched with E9 JMP");
            self.is_injected = true;
            return Ok(());
        }
        if prologue != [0x81, 0xEC, 0x88, 0x02, 0x00, 0x00] {
            return Err(format!(
                "GetItemName prologue mismatch at 0x{:X}: expected 81 EC 88 02 00 00, got {:02X?}",
                self.get_item_name_addr, prologue
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
            return Err("VirtualAllocEx failed for ItemExtraInfo trampoline".to_string());
        }
        self.trampoline_addr = trampoline_addr;

        let g_enabled = (trampoline_addr + DATA_OFFSET + OFFSET_ENABLED) as u32;
        let g_show_socks_eth = (trampoline_addr + DATA_OFFSET + OFFSET_SHOW_SOCKS_ETH) as u32;
        let g_d2common_get_stat = (trampoline_addr + DATA_OFFSET + OFFSET_D2COMMON_GET_STAT) as u32;
        let g_orig_resume = (trampoline_addr + DATA_OFFSET + OFFSET_ORIG_RESUME_ADDR) as u32;
        let g_temp_buf = (trampoline_addr + DATA_OFFSET + OFFSET_TEMP_BUF) as u32;

        // Build trampoline bytecode
        let mut code = Vec::with_capacity(512);

        // --- Entry point ---
        // mov eax, [g_enabled]
        code.push(0xA1);
        code.extend_from_slice(&g_enabled.to_le_bytes());
        // test eax, eax
        code.extend_from_slice(&[0x85, 0xC0]);
        // jnz .do_extra_info
        let do_extra_info_jmp_idx = code.len();
        code.extend_from_slice(&[0x0F, 0x85, 0, 0, 0, 0]);

        // .fallback: jmp .original_stub
        let fallback_jmp_stub_idx = code.len();
        code.extend_from_slice(&[0xE9, 0, 0, 0, 0]);

        // .do_extra_info:
        let do_extra_info_offset = code.len();
        let rel_do_extra =
            (do_extra_info_offset as isize - (do_extra_info_jmp_idx as isize + 6)) as i32;
        code[do_extra_info_jmp_idx + 2..do_extra_info_jmp_idx + 6]
            .copy_from_slice(&rel_do_extra.to_le_bytes());

        // push ebp; mov ebp, esp; push ebx; push esi; push edi
        code.extend_from_slice(&[0x55, 0x89, 0xE5, 0x53, 0x56, 0x57]);

        // Call .original_stub to let original function populate pBuffer
        // push [ebp + 0x10] (max_len)
        // push [ebp + 0x0C] (pBuffer)
        // push [ebp + 0x08] (pItem)
        code.extend_from_slice(&[
            0xFF, 0x75, 0x10, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08, 0xE8, 0, 0, 0, 0,
        ]);
        let call_stub_idx = code.len() - 4;

        // test eax, eax (check if original function returned TRUE)
        code.extend_from_slice(&[0x85, 0xC0]);
        let orig_failed_jmp_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz .return_done

        // mov esi, [ebp + 0x08] (ESI = pItem)
        code.extend_from_slice(&[0x8B, 0x75, 0x08]);
        // mov ebx, [ebp + 0x0C] (EBX = pBuffer)
        code.extend_from_slice(&[0x8B, 0x5D, 0x0C]);
        // mov edi, g_temp_buf   (EDI = g_temp_buf)
        code.push(0xBF);
        code.extend_from_slice(&g_temp_buf.to_le_bytes());

        // 1. Write "UID:0x" -> 'U'(0x55), 'I'(0x49), 'D'(0x44), ':'(0x3A), '0'(0x30), 'x'(0x78)
        code.extend_from_slice(&[
            0x66, 0xC7, 0x07, 0x55, 0x00, 0x66, 0xC7, 0x47, 0x02, 0x49, 0x00, 0x66, 0xC7, 0x47,
            0x04, 0x44, 0x00, 0x66, 0xC7, 0x47, 0x06, 0x3A, 0x00, 0x66, 0xC7, 0x47, 0x08, 0x30,
            0x00, 0x66, 0xC7, 0x47, 0x0A, 0x78, 0x00, 0x83, 0xC7, 0x0C,
        ]);

        // eax = [esi + 0x0C] (dwUnitId)
        code.extend_from_slice(&[0x8B, 0x46, 0x0C]);
        // call append_hex
        let call_append_hex_idx1 = code.len();
        code.extend_from_slice(&[0xE8, 0, 0, 0, 0]);

        // Write " CID:" -> ' '(0x20), 'C'(0x43), 'I'(0x49), 'D'(0x44), ':'(0x3A)
        code.extend_from_slice(&[
            0x66, 0xC7, 0x07, 0x20, 0x00, 0x66, 0xC7, 0x47, 0x02, 0x43, 0x00, 0x66, 0xC7, 0x47,
            0x04, 0x49, 0x00, 0x66, 0xC7, 0x47, 0x06, 0x44, 0x00, 0x66, 0xC7, 0x47, 0x08, 0x3A,
            0x00, 0x83, 0xC7, 0x0A,
        ]);

        // eax = [esi + 0x04] (dwClassId)
        code.extend_from_slice(&[0x8B, 0x46, 0x04]);
        // call append_dec
        let call_append_dec_idx1 = code.len();
        code.extend_from_slice(&[0xE8, 0, 0, 0, 0]);

        // Write "\n" -> 0x000A
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x0A, 0x00, 0x83, 0xC7, 0x02]);

        // 2. Copy original buffer contents to g_temp_buf
        // .copy_orig:
        let copy_orig_offset = code.len();
        code.extend_from_slice(&[0x66, 0x8B, 0x0B, 0x66, 0x85, 0xC9]);
        let copy_orig_done_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz .copy_orig_done
        code.extend_from_slice(&[0x66, 0x89, 0x0F, 0x83, 0xC3, 0x02, 0x83, 0xC7, 0x02, 0xEB]);
        let loop_back = (copy_orig_offset as isize - (code.len() as isize + 1)) as i8;
        code.push(loop_back as u8);

        // .copy_orig_done:
        let copy_orig_done_offset = code.len();
        code[copy_orig_done_idx + 1] = (copy_orig_done_offset - (copy_orig_done_idx + 2)) as u8;

        // 3. Check show_sockets_and_eth
        // mov eax, [g_show_socks_eth]
        code.push(0xA1);
        code.extend_from_slice(&g_show_socks_eth.to_le_bytes());
        // test eax, eax; jz .finish_string
        code.extend_from_slice(&[0x85, 0xC0]);
        let skip_socks_eth_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz

        // Check sockets: push 0; push 194; push esi; call dword ptr [g_d2common_get_stat]
        code.extend_from_slice(&[0x6A, 0x00, 0x68, 0xC2, 0x00, 0x00, 0x00, 0x56, 0xFF, 0x15]);
        code.extend_from_slice(&g_d2common_get_stat.to_le_bytes());
        // test eax, eax; jle .check_eth
        code.extend_from_slice(&[0x85, 0xC0]);
        let check_eth_jmp_idx = code.len();
        code.extend_from_slice(&[0x7E, 0]); // short jle

        // Write " (" -> 0x0020, 0x0028
        code.extend_from_slice(&[
            0x66, 0xC7, 0x07, 0x20, 0x00, 0x66, 0xC7, 0x47, 0x02, 0x28, 0x00, 0x83, 0xC7, 0x04,
        ]);
        // call append_dec (EAX is socks count)
        let call_append_dec_idx2 = code.len();
        code.extend_from_slice(&[0xE8, 0, 0, 0, 0]);
        // Write "s)" -> 0x0073, 0x0029
        code.extend_from_slice(&[
            0x66, 0xC7, 0x07, 0x73, 0x00, 0x66, 0xC7, 0x47, 0x02, 0x29, 0x00, 0x83, 0xC7, 0x04,
        ]);

        // .check_eth:
        let check_eth_offset = code.len();
        code[check_eth_jmp_idx + 1] = (check_eth_offset - (check_eth_jmp_idx + 2)) as u8;

        // pItemData = [esi + 0x14]
        code.extend_from_slice(&[0x8B, 0x46, 0x14, 0x85, 0xC0]);
        let no_item_data_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz

        // flags = [eax + 0x00]
        // test eax, 0x00400000 (ITEMFLAG_ETHEREAL)
        code.extend_from_slice(&[0x8B, 0x00, 0xA9, 0x00, 0x00, 0x40, 0x00]);
        let not_eth_idx = code.len();
        code.extend_from_slice(&[0x74, 0]); // short jz

        // Write " (eth)" -> 0x0020, 0x0028, 0x0065, 0x0074, 0x0068, 0x0029
        code.extend_from_slice(&[
            0x66, 0xC7, 0x07, 0x20, 0x00, 0x66, 0xC7, 0x47, 0x02, 0x28, 0x00, 0x66, 0xC7, 0x47,
            0x04, 0x65, 0x00, 0x66, 0xC7, 0x47, 0x06, 0x74, 0x00, 0x66, 0xC7, 0x47, 0x08, 0x68,
            0x00, 0x66, 0xC7, 0x47, 0x0A, 0x29, 0x00, 0x83, 0xC7, 0x0C,
        ]);

        let not_eth_offset = code.len();
        code[no_item_data_idx + 1] = (not_eth_offset - (no_item_data_idx + 2)) as u8;
        code[not_eth_idx + 1] = (not_eth_offset - (not_eth_idx + 2)) as u8;

        // .finish_string:
        let finish_str_offset = code.len();
        code[skip_socks_eth_idx + 1] = (finish_str_offset - (skip_socks_eth_idx + 2)) as u8;

        // Null terminator: mov word ptr [edi], 0x0000
        code.extend_from_slice(&[0x66, 0xC7, 0x07, 0x00, 0x00]);

        // 4. Copy g_temp_buf back into pBuffer: [ebp + 0x0C]
        code.push(0xBE);
        code.extend_from_slice(&g_temp_buf.to_le_bytes());
        code.extend_from_slice(&[0x8B, 0x7D, 0x0C]); // mov edi, [ebp + 0x0C]

        // .copy_back_loop:
        let copy_back_offset = code.len();
        code.extend_from_slice(&[
            0x66, 0x8B, 0x0E, 0x66, 0x89, 0x0F, 0x83, 0xC6, 0x02, 0x83, 0xC7, 0x02, 0x66, 0x85,
            0xC9,
        ]);
        let cb_loop_back = (copy_back_offset as isize - (code.len() as isize + 2)) as i8;
        code.extend_from_slice(&[0x75, cb_loop_back as u8]);

        // mov eax, 1 (TRUE)
        code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]);

        // .return_done:
        let ret_done_offset = code.len();
        code[orig_failed_jmp_idx + 1] = (ret_done_offset - (orig_failed_jmp_idx + 2)) as u8;

        code.extend_from_slice(&[0x5F, 0x5E, 0x5B, 0x89, 0xEC, 0x5D, 0xC2, 0x0C, 0x00]); // ret 0x0C

        // --- SUBROUTINES ---

        // Subroutine: original_stub (executes replaced instruction `sub esp, 288h` and jumps to resume)
        let sub_orig_stub_offset = code.len();
        // sub esp, 0x0288: 81 EC 88 02 00 00
        code.extend_from_slice(&[0x81, 0xEC, 0x88, 0x02, 0x00, 0x00]);
        // jmp dword ptr [g_orig_resume]: FF 25 [g_orig_resume]
        code.extend_from_slice(&[0xFF, 0x25]);
        code.extend_from_slice(&g_orig_resume.to_le_bytes());

        // Fixup fallback_jmp_stub
        let rel_stub =
            (sub_orig_stub_offset as isize - (fallback_jmp_stub_idx as isize + 5)) as i32;
        code[fallback_jmp_stub_idx + 1..fallback_jmp_stub_idx + 5]
            .copy_from_slice(&rel_stub.to_le_bytes());

        // Fixup call_stub_idx
        let rel_call_stub = (sub_orig_stub_offset as isize - (call_stub_idx as isize + 4)) as i32;
        code[call_stub_idx..call_stub_idx + 4].copy_from_slice(&rel_call_stub.to_le_bytes());

        // Subroutine: append_dec (EAX = u32, writes decimal to EDI)
        let sub_append_dec_offset = code.len();
        code.extend_from_slice(&[
            0x53, 0x56, 0xBB, 0x0A, 0x00, 0x00, 0x00, 0x31, 0xC9, 0x31, 0xD2, 0xF7, 0xF3, 0x52,
            0x41, 0x85, 0xC0, 0x75, 0xF6, 0x5A, 0x83, 0xC2, 0x30, 0x66, 0x89, 0x17, 0x83, 0xC7,
            0x02, 0x49, 0x75, 0xF2, 0x5E, 0x5B, 0xC3,
        ]);

        // Subroutine: append_hex (EAX = u32, writes hexadecimal to EDI)
        let sub_append_hex_offset = code.len();
        code.extend_from_slice(&[
            0x53, 0x56, 0xBB, 0x10, 0x00, 0x00, 0x00, 0x31, 0xC9, 0x31, 0xD2, 0xF7, 0xF3, 0x52,
            0x41, 0x85, 0xC0, 0x75, 0xF6, 0x5A, 0x83, 0xFA, 0x0A, 0x7C, 0x05, 0x83, 0xC2, 0x37,
            0xEB, 0x03, 0x83, 0xC2, 0x30, 0x66, 0x89, 0x17, 0x83, 0xC7, 0x02, 0x49, 0x75, 0xE9,
            0x5E, 0x5B, 0xC3,
        ]);

        // Fix up CALL rel32 targets
        let fixup_call = |code: &mut Vec<u8>, call_site_idx: usize, target_offset: usize| {
            let next_ip = call_site_idx + 5;
            let rel = (target_offset as isize - next_ip as isize) as i32;
            code[call_site_idx + 1..call_site_idx + 5].copy_from_slice(&rel.to_le_bytes());
        };

        fixup_call(&mut code, call_append_dec_idx1, sub_append_dec_offset);
        fixup_call(&mut code, call_append_dec_idx2, sub_append_dec_offset);
        fixup_call(&mut code, call_append_hex_idx1, sub_append_hex_offset);

        // 3. Write trampoline code to remote memory
        process.write_buffer(trampoline_addr, &code)?;

        // 4. Initialize trampoline data
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_ENABLED,
            &(self.enabled as u32).to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_SHOW_SOCKS_ETH,
            &(self.show_sockets_and_eth as u32).to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_D2COMMON_GET_STAT,
            &d2common_get_stat_addr.to_le_bytes(),
        )?;
        process.write_buffer(
            trampoline_addr + DATA_OFFSET + OFFSET_ORIG_RESUME_ADDR,
            &orig_resume_addr.to_le_bytes(),
        )?;

        // 5. Patch GetItemName entry point with E9 rel32 + NOP (6 bytes total)
        let rel_hook = (trampoline_addr as isize - (self.get_item_name_addr as isize + 5)) as i32;
        let mut patch_bytes = [0u8; 6];
        patch_bytes[0] = 0xE9;
        patch_bytes[1..5].copy_from_slice(&rel_hook.to_le_bytes());
        patch_bytes[5] = 0x90; // NOP to match 6-byte instruction boundary

        #[cfg(target_os = "windows")]
        unsafe {
            let mut old_protect = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                process.handle,
                self.get_item_name_addr as *const c_void,
                6,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
            .map_err(|e| {
                format!(
                    "VirtualProtectEx RWX failed at 0x{:X}: {}",
                    self.get_item_name_addr, e
                )
            })?;

            process.write_buffer(self.get_item_name_addr, &patch_bytes)?;

            let mut temp = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                process.handle,
                self.get_item_name_addr as *const c_void,
                6,
                old_protect,
                &mut temp,
            )
            .map_err(|e| {
                format!(
                    "VirtualProtectEx restore failed at 0x{:X}: {}",
                    self.get_item_name_addr, e
                )
            })?;

            let _ = FlushInstructionCache(
                process.handle,
                Some(self.get_item_name_addr as *const c_void),
                6,
            );
        }
        #[cfg(not(target_os = "windows"))]
        process.write_buffer(self.get_item_name_addr, &patch_bytes)?;

        self.is_injected = true;
        crate::logger::info(&format!(
            "ItemExtraInfoHook: Injected successfully! trampoline=0x{:08X}, hook_point=0x{:08X}",
            trampoline_addr, self.get_item_name_addr
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

    /// Dynamically update show_sockets_and_eth flag in remote memory
    pub fn set_show_sockets_and_eth(
        &mut self,
        process: &ProcessHandle,
        show: bool,
    ) -> Result<(), String> {
        self.show_sockets_and_eth = show;
        if self.is_injected && self.trampoline_addr != 0 {
            process.write_buffer(
                self.trampoline_addr + DATA_OFFSET + OFFSET_SHOW_SOCKS_ETH,
                &(show as u32).to_le_bytes(),
            )?;
        }
        Ok(())
    }
}
