//! Auto Belt Module
//! Automatically replenishes empty belt slots from potions in the player's inventory
//! by sending native game packets (packet 0x63 ItemToBelt).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::injection::D2Injector;
use crate::offsets::{
    body_loc, d2client, d2common, inventory, inventory_grid, item_data, items_txt, unit,
};
use crate::process::D2Context;

const ACTION_COOLDOWN_MS: u64 = 250;

/// Well-known Median XL potion 4-character codes from misc.txt
const KNOWN_POTION_CODES: [&[u8; 4]; 20] = [
    b"hpo ", b"mpo ", b"hpf ", b"mpf ", b"wms ", b"hrt ", b"rvs ", b"rvl ", b"hp1 ", b"hp2 ",
    b"hp3 ", b"hp4 ", b"hp5 ", b"mp1 ", b"mp2 ", b"mp3 ", b"mp4 ", b"mp5 ", b"yps ", b"vps ",
];

pub struct AutoBeltState {
    pub enabled: AtomicBool,
    last_action: Mutex<Instant>,
}

impl AutoBeltState {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            last_action: Mutex::new(Instant::now() - Duration::from_secs(5)),
        }
    }

    /// Periodic tick executed during scanner loop
    pub fn tick(&self, ctx: &D2Context, injector: &Mutex<D2Injector>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        // Throttle actions to avoid flooding packets before server state updates
        {
            let last = self.last_action.lock().unwrap();
            if last.elapsed() < Duration::from_millis(ACTION_COOLDOWN_MS) {
                return;
            }
        }

        // 1. Check if the belt has at least one empty slot
        if !has_empty_belt_slot(ctx) {
            return;
        }

        // 2. Check for a potion in the player's inventory
        if let Some(inv_uid) = find_inventory_potion(ctx) {
            let packet = build_inventory_to_belt_packet(inv_uid);
            if let Ok(inj) = injector.lock() {
                if let Err(e) = inj.send_packet(&ctx.process, &packet) {
                    crate::logger::error(&format!("AutoBelt: inventory move packet failed: {}", e));
                } else {
                    crate::logger::info(&format!(
                        "AutoBelt: moved inventory potion to belt (ID: 0x{:X})",
                        inv_uid
                    ));
                    if let Ok(mut last) = self.last_action.lock() {
                        *last = Instant::now();
                    }
                }
            }
        }
    }
}

/// Checks whether an item's class ID represents a potion that can enter the belt
pub fn is_potion(ctx: &D2Context, class_id: u32) -> bool {
    let items_base = match ctx
        .process
        .read_memory::<u32>(ctx.d2_common + d2common::ITEMS_TXT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };
    let items_count = ctx
        .process
        .read_memory::<u32>(ctx.d2_common + d2common::ITEMS_TXT_COUNT)
        .unwrap_or(0);

    if class_id == 0 || class_id >= items_count {
        return false;
    }

    let record = items_base + class_id as usize * items_txt::RECORD_SIZE;

    // Check autobelt flag in Items.txt record (+0x199)
    if let Ok(autobelt) = ctx.process.read_memory::<u8>(record + items_txt::AUTOBELT) {
        if autobelt == 1 {
            return true;
        }
    }

    // Check 4-character code at +0x74
    if let Ok(code_bytes) = ctx.process.read_memory::<[u8; 4]>(record + items_txt::CODE) {
        if KNOWN_POTION_CODES.contains(&&code_bytes) {
            return true;
        }
    }

    false
}

/// Returns true if the player's belt currently has at least one open slot
pub fn has_empty_belt_slot(ctx: &D2Context) -> bool {
    let p_player = match ctx
        .process
        .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    let p_inv = match ctx.process.read_memory::<u32>(p_player + unit::INVENTORY) {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    let p_grids = match ctx.process.read_memory::<u32>(p_inv + inventory::GRIDS) {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    // Determine belt capacity:
    // If an equipped belt exists in BodyLoc grid (grid 0, index 8), Median XL belts have 16 slots.
    // If no belt is equipped, base character belt has 4 slots (1 row).
    let mut max_capacity = 4usize;
    if let Ok(grid0_pp_items) = ctx
        .process
        .read_memory::<u32>(p_grids + inventory_grid::PP_ITEMS)
    {
        if grid0_pp_items != 0 {
            let equipped_belt = ctx
                .process
                .read_memory::<u32>(grid0_pp_items as usize + body_loc::BELT * 4)
                .unwrap_or(0);
            if equipped_belt != 0 {
                max_capacity = 16;
            }
        }
    }

    // Also check Grid 1 (INVGRID_BELT at p_grids + 0x10)
    let p_belt_grid = p_grids + inventory_grid::SIZE;
    let grid_x = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x00)
        .unwrap_or(4);
    let grid_y = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x04)
        .unwrap_or(0);

    if grid_x > 0 && grid_y > 0 {
        let grid_cap = ((grid_x * grid_y) as usize).clamp(4, 16);
        max_capacity = grid_cap;

        // Check if any slot pointer in the belt grid is NULL
        if let Ok(pp_items) = ctx
            .process
            .read_memory::<u32>(p_belt_grid + inventory_grid::PP_ITEMS)
        {
            if pp_items != 0 {
                for slot in 0..grid_cap {
                    let p_item = ctx
                        .process
                        .read_memory::<u32>(pp_items as usize + slot * 4)
                        .unwrap_or(0);
                    if p_item == 0 {
                        return true;
                    }
                }
            }
        }
    }

    // Secondary fallback check: count items owned by the player with ITEM_LOCATION == 2 (BELT)
    let mut p_item = match ctx
        .process
        .read_memory::<u32>(p_inv + inventory::FIRST_ITEM)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    let mut belt_items_count = 0usize;
    for _ in 0..256 {
        if p_item == 0 {
            break;
        }

        let p_unit_data = match ctx.process.read_memory::<u32>(p_item + unit::UNIT_DATA) {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };

        let item_loc = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::ITEM_LOCATION)
            .unwrap_or(0xFF);

        if item_loc == 2 {
            belt_items_count += 1;
        }

        p_item = match ctx
            .process
            .read_memory::<u32>(p_unit_data + item_data::NEXT_ITEM)
        {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };
    }

    belt_items_count < max_capacity
}

/// Finds a potion in the player's main inventory to move into the belt
pub fn find_inventory_potion(ctx: &D2Context) -> Option<u32> {
    let p_player = match ctx
        .process
        .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let p_inv = match ctx.process.read_memory::<u32>(p_player + unit::INVENTORY) {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let mut p_item = match ctx
        .process
        .read_memory::<u32>(p_inv + inventory::FIRST_ITEM)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    // Cap iterations to avoid infinite loop on cycle
    for _ in 0..256 {
        if p_item == 0 {
            break;
        }

        let p_unit_data = match ctx.process.read_memory::<u32>(p_item + unit::UNIT_DATA) {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };

        // ITEM_LOCATION = 0 (stored), GAME_LOCATION = 3 (inventory)
        let item_loc = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::ITEM_LOCATION)
            .unwrap_or(0xFF);
        let game_loc = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::GAME_LOCATION)
            .unwrap_or(0xFF);

        if item_loc == 0 && game_loc == 3 {
            let class_id = ctx
                .process
                .read_memory::<u32>(p_item + unit::CLASS)
                .unwrap_or(0);
            if is_potion(ctx, class_id) {
                if let Ok(unit_id) = ctx.process.read_memory::<u32>(p_item + unit::UNIT_ID) {
                    return Some(unit_id);
                }
            }
        }

        p_item = match ctx
            .process
            .read_memory::<u32>(p_unit_data + item_data::NEXT_ITEM)
        {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };
    }

    None
}

/// Builds the 5-byte 0x63 packet (ItemToBelt) to move an inventory item directly to belt
pub fn build_inventory_to_belt_packet(unit_id: u32) -> [u8; 5] {
    let mut packet = [0u8; 5];
    packet[0] = 0x63; // D2GS_ITEMTOBELT
    packet[1..5].copy_from_slice(&unit_id.to_le_bytes());
    packet
}

/// Tauri command to toggle auto belt
#[tauri::command]
pub fn toggle_auto_belt(
    state: tauri::State<'_, AutoBeltState>,
    enabled: bool,
) -> Result<(), String> {
    state.enabled.store(enabled, Ordering::Relaxed);
    Ok(())
}
