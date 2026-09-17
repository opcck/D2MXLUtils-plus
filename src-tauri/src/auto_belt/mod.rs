//! Auto Belt Module
//! Automatically replenishes empty belt slots from potions in the player's inventory
//! by sending native game packets (packet 0x63 ItemToBelt).
//!
//! Aligned with Hackmap's auto_item_to_belt logic:
//! 1. Checks if the player is holding an item on the cursor (safety check).
//! 2. Inspects belt grid (Grid 1) slots and columns to determine acceptable potion families.
//! 3. Iterates player inventory items with inv_page == 0 (D2ItemInvPage::Inventory).
//! 4. Maintains an in-flight cooldown map (items_removing) to prevent duplicate packets.
//! 5. Dispatches packet 0x63 with an action throttle.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::injection::D2Injector;
use crate::offsets::{d2client, d2common, inventory, inventory_grid, item_data, items_txt, unit};
use crate::process::D2Context;

const ACTION_COOLDOWN_MS: u64 = 400;
const ITEM_IN_FLIGHT_SECS: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionFamily {
    Healing,
    Mana,
    Rejuvenation,
    Utility,
    ScrollTp,
    ScrollId,
    Other,
}

pub struct AutoBeltState {
    pub enabled: AtomicBool,
    last_action: Mutex<Instant>,
    items_removing: Mutex<HashMap<u32, Instant>>,
}

impl AutoBeltState {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            last_action: Mutex::new(Instant::now() - Duration::from_secs(5)),
            items_removing: Mutex::new(HashMap::new()),
        }
    }

    pub fn clear(&self) {
        if let Ok(mut removing) = self.items_removing.lock() {
            removing.clear();
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

        // Clean expired in-flight entries
        {
            let now = Instant::now();
            if let Ok(mut removing) = self.items_removing.lock() {
                removing.retain(|_, expire_time| *expire_time > now);
            }
        }

        // Find next candidate potion in inventory that matches an empty slot in belt
        let candidate = {
            let removing = self.items_removing.lock().unwrap();
            find_next_belt_replenish_item(ctx, &removing)
        };

        if let Some((inv_uid, family)) = candidate {
            let packet = build_inventory_to_belt_packet(inv_uid);
            if let Ok(inj) = injector.lock() {
                match inj.send_packet(&ctx.process, &packet) {
                    Ok(_) => {
                        crate::logger::info(&format!(
                            "AutoBelt: replenished {:?} potion to belt (GUID: 0x{:X})",
                            family, inv_uid
                        ));
                        if let Ok(mut removing) = self.items_removing.lock() {
                            removing.insert(
                                inv_uid,
                                Instant::now() + Duration::from_secs(ITEM_IN_FLIGHT_SECS),
                            );
                        }
                        if let Ok(mut last) = self.last_action.lock() {
                            *last = Instant::now();
                        }
                    }
                    Err(e) => {
                        crate::logger::error(&format!(
                            "AutoBelt: packet 0x63 failed for item 0x{:X}: {}",
                            inv_uid, e
                        ));
                    }
                }
            }
        }
    }
}

/// Determines the PotionFamily of an item by reading its class and Items.txt record
pub fn get_item_potion_family(ctx: &D2Context, p_item: usize) -> Option<PotionFamily> {
    let class_id = ctx
        .process
        .read_memory::<u32>(p_item + unit::CLASS)
        .unwrap_or(0);
    let items_base = match ctx
        .process
        .read_memory::<u32>(ctx.d2_common + d2common::ITEMS_TXT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };
    let items_count = ctx
        .process
        .read_memory::<u32>(ctx.d2_common + d2common::ITEMS_TXT_COUNT)
        .unwrap_or(0);

    if class_id == 0 || class_id >= items_count {
        return None;
    }

    let record = items_base + class_id as usize * items_txt::RECORD_SIZE;

    let code = ctx
        .process
        .read_memory::<[u8; 4]>(record + items_txt::CODE)
        .unwrap_or([0; 4]);
    let type_0 = ctx
        .process
        .read_memory::<u16>(record + items_txt::TYPE_0)
        .unwrap_or(0);
    let autobelt = ctx
        .process
        .read_memory::<u8>(record + items_txt::AUTOBELT)
        .unwrap_or(0);
    let belt = ctx
        .process
        .read_memory::<u8>(record + items_txt::BELT)
        .unwrap_or(0);

    // Type 76 = HealingPotion, 77 = ManaPotion, 78 = RejuvPotion, 79 = Stamina, 80 = Antidote, 81 = Thawing
    if type_0 == 76
        || code.starts_with(b"hp")
        || &code == b"hpo "
        || &code == b"hpf "
        || &code == b"b@b "
    {
        return Some(PotionFamily::Healing);
    }
    if type_0 == 77 || code.starts_with(b"mp") || &code == b"mpo " || &code == b"mpf " {
        return Some(PotionFamily::Mana);
    }
    if type_0 == 78 || &code == b"rvs " || &code == b"rvl " || &code == b"dog " {
        return Some(PotionFamily::Rejuvenation);
    }
    if type_0 == 79
        || type_0 == 80
        || type_0 == 81
        || &code == b"wms "
        || &code == b"yps "
        || &code == b"vps "
    {
        return Some(PotionFamily::Utility);
    }
    if &code == b"tsc " {
        return Some(PotionFamily::ScrollTp);
    }
    if &code == b"isc " {
        return Some(PotionFamily::ScrollId);
    }
    if autobelt == 1 || belt == 1 {
        return Some(PotionFamily::Other);
    }

    None
}

/// Finds the next inventory potion that can be placed into the player's belt
pub fn find_next_belt_replenish_item(
    ctx: &D2Context,
    items_in_flight: &HashMap<u32, Instant>,
) -> Option<(u32, PotionFamily)> {
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

    // 1. If player has an item held on cursor, do not move belt items
    if let Ok(cursor_item) = ctx
        .process
        .read_memory::<u32>(p_inv + inventory::CURSOR_ITEM)
    {
        if cursor_item != 0 {
            return None;
        }
    }

    // 2. Read belt grid (Grid 1 at p_grids + 0x10)
    let p_grids = match ctx.process.read_memory::<u32>(p_inv + inventory::GRIDS) {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let p_belt_grid = p_grids + inventory_grid::SIZE;
    let grid_x = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x00)
        .unwrap_or(4)
        .clamp(1, 4) as usize;
    let grid_y = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x04)
        .unwrap_or(1)
        .clamp(1, 4) as usize;
    let total_slots = grid_x * grid_y;

    let pp_items = match ctx
        .process
        .read_memory::<u32>(p_belt_grid + inventory_grid::PP_ITEMS)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    let mut belt_slots = Vec::with_capacity(total_slots);
    let mut has_empty_slot = false;
    for i in 0..total_slots {
        let item_ptr = ctx
            .process
            .read_memory::<u32>(pp_items + i * 4)
            .unwrap_or(0) as usize;
        if item_ptr == 0 {
            has_empty_slot = true;
        }
        belt_slots.push(item_ptr);
    }

    // If belt has zero empty slots, nothing to replenish
    if !has_empty_slot {
        return None;
    }

    // 3. Analyze belt columns to see which potion families can be accepted
    let mut empty_column_exists = false;
    let mut acceptable_families = Vec::new();

    for col in 0..grid_x {
        let bottom_item = belt_slots[col];
        if bottom_item == 0 {
            empty_column_exists = true;
        } else if let Some(family) = get_item_potion_family(ctx, bottom_item) {
            // Check if this column has an empty row above row 0
            for row in 1..grid_y {
                let slot_idx = col + row * 4;
                if slot_idx < total_slots && belt_slots[slot_idx] == 0 {
                    if !acceptable_families.contains(&family) {
                        acceptable_families.push(family);
                    }
                    break;
                }
            }
        }
    }

    if !empty_column_exists && acceptable_families.is_empty() {
        return None;
    }

    // 4. Scan inventory linked list for matching potions
    let mut p_item = match ctx
        .process
        .read_memory::<u32>(p_inv + inventory::FIRST_ITEM)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };

    for _ in 0..256 {
        if p_item == 0 {
            break;
        }

        let u_type = ctx
            .process
            .read_memory::<u32>(p_item + unit::UNIT_TYPE)
            .unwrap_or(0);
        let p_unit_data = match ctx.process.read_memory::<u32>(p_item + unit::UNIT_DATA) {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };

        if u_type != 4 {
            break;
        }

        let next_item = ctx
            .process
            .read_memory::<u32>(p_unit_data + item_data::NEXT_ITEM)
            .unwrap_or(0) as usize;

        // INV_PAGE == 0 means main inventory (0=inventory, 1=equip, 2=trade, 3=cube, 4=stash, 5=belt)
        let inv_page = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::INV_PAGE)
            .unwrap_or(0xFF);

        if inv_page == 0 {
            if let Ok(unit_id) = ctx.process.read_memory::<u32>(p_item + unit::UNIT_ID) {
                if !items_in_flight.contains_key(&unit_id) {
                    if let Some(family) = get_item_potion_family(ctx, p_item) {
                        if empty_column_exists || acceptable_families.contains(&family) {
                            return Some((unit_id, family));
                        }
                    }
                }
            }
        }

        p_item = next_item;
    }

    None
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

    if let Ok(autobelt) = ctx.process.read_memory::<u8>(record + items_txt::AUTOBELT) {
        if autobelt == 1 {
            return true;
        }
    }
    if let Ok(belt) = ctx.process.read_memory::<u8>(record + items_txt::BELT) {
        if belt == 1 {
            return true;
        }
    }

    if let Ok(code) = ctx.process.read_memory::<[u8; 4]>(record + items_txt::CODE) {
        if code.starts_with(b"hp")
            || code.starts_with(b"mp")
            || &code == b"rvs "
            || &code == b"rvl "
        {
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

    let p_belt_grid = p_grids + inventory_grid::SIZE;
    let grid_x = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x00)
        .unwrap_or(4)
        .clamp(1, 4) as usize;
    let grid_y = ctx
        .process
        .read_memory::<i32>(p_belt_grid + 0x04)
        .unwrap_or(1)
        .clamp(1, 4) as usize;
    let total_slots = grid_x * grid_y;

    let pp_items = match ctx
        .process
        .read_memory::<u32>(p_belt_grid + inventory_grid::PP_ITEMS)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return false,
    };

    for i in 0..total_slots {
        let p_item = ctx
            .process
            .read_memory::<u32>(pp_items + i * 4)
            .unwrap_or(0);
        if p_item == 0 {
            return true;
        }
    }

    false
}

/// Finds a potion in the player's main inventory to move into the belt
pub fn find_inventory_potion(ctx: &D2Context) -> Option<u32> {
    let empty_map = HashMap::new();
    find_next_belt_replenish_item(ctx, &empty_map).map(|(id, _)| id)
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
    crate::logger::info(&format!("AutoBelt toggled: enabled = {}", enabled));
    Ok(())
}
