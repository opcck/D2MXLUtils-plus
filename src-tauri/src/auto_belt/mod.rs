//! Auto Belt Module
//! Automatically replenishes empty belt slots from nearby ground potions (within ~5 yards)
//! or from potions in the player's inventory by sending native game packets.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::injection::D2Injector;
use crate::map_markers::manager::{bfs_item_positions, read_player_subtile};
use crate::offsets::{
    d2client, d2common, inventory, inventory_grid, item_data, items_txt, unit, unit_type,
};
use crate::process::D2Context;

/// Maximum range in subtiles for picking up ground potions (~5 yards).
/// In Diablo II, 1 yard is approximately 3 subtiles -> 5 yards ≈ 15 subtiles.
const GROUND_PICKUP_RANGE_SUBTILES: i32 = 16;
const ACTION_COOLDOWN_MS: u64 = 300;

/// Well-known Median XL potion 4-character codes from misc.txt
const KNOWN_POTION_CODES: [&[u8; 4]; 19] = [
    b"hpo ", b"mpo ", b"hpf ", b"mpf ", b"wms ", b"hrt ", b"rvs ", b"rvl ", b"hp1 ", b"hp2 ",
    b"hp3 ", b"hp4 ", b"hp5 ", b"mp1 ", b"mp2 ", b"mp3 ", b"mp4 ", b"mp5 ", b"yps ",
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

        // 2. Try picking up a ground potion within 5 yards
        if let Some(ground_uid) = find_nearby_ground_potion(ctx, GROUND_PICKUP_RANGE_SUBTILES) {
            let packet = build_ground_to_belt_packet(ground_uid);
            if let Ok(inj) = injector.lock() {
                if let Err(e) = inj.send_packet(&ctx.process, &packet) {
                    crate::logger::error(&format!("AutoBelt: ground pickup packet failed: {}", e));
                } else {
                    crate::logger::info(&format!(
                        "AutoBelt: picked up ground potion (ID: {})",
                        ground_uid
                    ));
                    if let Ok(mut last) = self.last_action.lock() {
                        *last = Instant::now();
                    }
                    return;
                }
            }
        }

        // 3. If no ground potion was found, check for a potion in the inventory
        if let Some(inv_uid) = find_inventory_potion(ctx) {
            let packet = build_inventory_to_belt_packet(inv_uid);
            if let Ok(inj) = injector.lock() {
                if let Err(e) = inj.send_packet(&ctx.process, &packet) {
                    crate::logger::error(&format!("AutoBelt: inventory move packet failed: {}", e));
                } else {
                    crate::logger::info(&format!(
                        "AutoBelt: moved inventory potion to belt (ID: {})",
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

    // Grid 1 is INVGRID_BELT (starts at p_grids + 0x10)
    let p_belt_grid = p_grids + inventory_grid::SIZE;
    let width = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x04)
        .unwrap_or(4);
    let height = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x08)
        .unwrap_or(1);

    let total_slots = if width > 0 && height > 0 {
        ((width * height) as usize).min(16)
    } else {
        4
    };

    let pp_items = match ctx
        .process
        .read_memory::<u32>(p_belt_grid + inventory_grid::PP_ITEMS)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    // Check each belt slot pointer
    for slot in 0..total_slots {
        let p_item = ctx
            .process
            .read_memory::<u32>(pp_items + slot * 4)
            .unwrap_or(0);
        if p_item == 0 {
            return true;
        }
    }

    false
}

/// Finds a ground potion within max_subtiles Manhattan/Euclidean distance from the player
pub fn find_nearby_ground_potion(ctx: &D2Context, max_subtiles: i32) -> Option<u32> {
    let (px, py) = read_player_subtile(ctx)?;
    let max_dist_sq = max_subtiles * max_subtiles;

    // Use Room1 BFS with small hop count (depth 3 is ample for ~5 yards)
    let positions = bfs_item_positions(ctx, 3).ok()?;

    for (p_unit, sx, sy) in positions {
        let dx = px - sx;
        let dy = py - sy;
        if dx * dx + dy * dy <= max_dist_sq {
            let class_id = match ctx
                .process
                .read_memory::<u32>(p_unit as usize + unit::CLASS)
            {
                Ok(c) => c,
                Err(_) => continue,
            };
            if is_potion(ctx, class_id) {
                if let Ok(unit_id) = ctx
                    .process
                    .read_memory::<u32>(p_unit as usize + unit::UNIT_ID)
                {
                    return Some(unit_id);
                }
            }
        }
    }

    None
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

/// Builds the 13-byte 0x16 packet to pick up a ground item directly to belt
pub fn build_ground_to_belt_packet(unit_id: u32) -> [u8; 13] {
    let mut packet = [0u8; 13];
    packet[0] = 0x16;
    packet[1] = 0x04; // UNIT_ITEM
    packet[5..9].copy_from_slice(&unit_id.to_le_bytes());
    packet
}

/// Builds the 9-byte 0x26 packet to move an inventory item directly to belt
pub fn build_inventory_to_belt_packet(unit_id: u32) -> [u8; 9] {
    let mut packet = [0u8; 9];
    packet[0] = 0x26;
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
