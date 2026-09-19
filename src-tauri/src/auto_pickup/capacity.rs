//! Hidden capacity and container validation for Auto Pickup.
//!
//! Provides non-intrusive, silent checks for:
//! 1. Belt capacity (for auto-filling potion slots).
//! 2. Horadric Cube presence and capacity.
//! 3. Main inventory space (to prevent attempting pickup when full).

use crate::offsets::{
    d2client, inventory, inventory_grid, item_data, item_path, items_txt, paths, unit,
};
use crate::process::D2Context;

/// Result of checking whether a container has room for an item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerSpace {
    Available,
    Full,
    ContainerNotFound,
}

/// Checks if player's belt has at least one empty potion slot
pub fn check_belt_has_empty_slot(ctx: &D2Context) -> bool {
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

    // Grid 1 is INVGRID_BELT
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

/// Checks whether an item's class ID represents the Horadric Cube.
/// Handles Median XL specific class ID (1204), string code ("box"), and 4-byte code.
pub fn is_horadric_cube(ctx: &D2Context, class_id: u32) -> bool {
    // 1. Median XL Horadric Cube class_id is 1204
    if class_id == 1204 {
        return true;
    }
    // 2. Base code string from Items.txt (checks 0x80 and 0x74)
    if let Some(s) = crate::inspector::read_item_code_string(ctx, class_id) {
        if s.eq_ignore_ascii_case("box") {
            return true;
        }
    }
    // 3. Raw 4-byte code from Items.txt (little-endian byte prefix)
    if let Some(code) = read_item_code(ctx, class_id) {
        let b = code.to_le_bytes();
        if &b[..3] == b"box" {
            return true;
        }
    }
    false
}

/// Finds the Horadric Cube unit ID in the player's inventory
pub fn find_horadric_cube(ctx: &D2Context) -> Option<u32> {
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
        Ok(p) => p,
        Err(_) => return None,
    };

    // Follow pFirstItem chain
    for _ in 0..256 {
        if p_item == 0 {
            break;
        }

        let p_unit_data = match ctx
            .process
            .read_memory::<u32>(p_item as usize + unit::UNIT_DATA)
        {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };

        let inv_page = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::INV_PAGE)
            .unwrap_or(255);

        let class_id = ctx
            .process
            .read_memory::<u32>(p_item as usize + unit::CLASS)
            .unwrap_or(0);
        let uid = ctx
            .process
            .read_memory::<u32>(p_item as usize + unit::UNIT_ID)
            .unwrap_or(0);

        // Cube must be in inventory (page 0) or stash (page 4)
        if (inv_page == 0 || inv_page == 4 || inv_page == 255) && is_horadric_cube(ctx, class_id) {
            if uid != 0 {
                return Some(uid);
            }
        }

        p_item = match ctx
            .process
            .read_memory::<u32>(p_unit_data + item_data::NEXT_ITEM)
        {
            Ok(p) => p,
            _ => break,
        };
    }

    None
}

/// Reads the 4-char code of an item class from Items.txt
pub fn read_item_code(ctx: &D2Context, class_id: u32) -> Option<u32> {
    let items_base = match ctx
        .process
        .read_memory::<u32>(ctx.d2_common + crate::offsets::d2common::ITEMS_TXT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return None,
    };
    let items_count = ctx
        .process
        .read_memory::<u32>(ctx.d2_common + crate::offsets::d2common::ITEMS_TXT_COUNT)
        .unwrap_or(0);

    if items_count > 0 && class_id >= items_count {
        return None;
    }

    let record_addr = items_base + (class_id as usize) * items_txt::RECORD_SIZE;
    if let Ok(code) = ctx
        .process
        .read_memory::<u32>(record_addr + items_txt::CODE)
    {
        if code != 0 {
            return Some(code);
        }
    }
    ctx.process.read_memory::<u32>(record_addr + 0x74).ok()
}

/// Reads inventory width & height (in grid cells) for a given class ID
pub fn read_item_dimensions(ctx: &D2Context, class_id: u32) -> (usize, usize) {
    let items_base = match ctx
        .process
        .read_memory::<u32>(ctx.d2_common + crate::offsets::d2common::ITEMS_TXT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return (1, 1),
    };
    let items_count = ctx
        .process
        .read_memory::<u32>(ctx.d2_common + crate::offsets::d2common::ITEMS_TXT_COUNT)
        .unwrap_or(0);

    if items_count > 0 && class_id >= items_count {
        return (1, 1);
    }

    let record_addr = items_base + (class_id as usize) * items_txt::RECORD_SIZE;
    let width = ctx
        .process
        .read_memory::<u8>(record_addr + items_txt::INV_WIDTH)
        .unwrap_or(1)
        .clamp(1, 4) as usize;
    let height = ctx
        .process
        .read_memory::<u8>(record_addr + items_txt::INV_HEIGHT)
        .unwrap_or(1)
        .clamp(1, 4) as usize;

    (width, height)
}

/// Checks whether the player's main inventory (Grid 2) has space for an item of size (width, height)
pub fn check_inventory_has_space(ctx: &D2Context, width: usize, height: usize) -> bool {
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

    // Grid 2 is INVGRID_INVENTORY
    let p_inv_grid = p_grids + 2 * inventory_grid::SIZE;
    let grid_x = ctx
        .process
        .read_memory::<i32>(p_inv_grid + 0x00)
        .unwrap_or(10)
        .clamp(1, 20) as usize;
    let grid_y = ctx
        .process
        .read_memory::<i32>(p_inv_grid + 0x04)
        .unwrap_or(8)
        .clamp(1, 20) as usize;

    let pp_items = match ctx
        .process
        .read_memory::<u32>(p_inv_grid + inventory_grid::PP_ITEMS)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return true, // If grid ptr read fails, default to true
    };

    // Read full grid into a 2D boolean array (occupied vs free)
    let total_cells = grid_x * grid_y;
    let mut occupied = vec![false; total_cells];

    for i in 0..total_cells {
        let p_item = ctx
            .process
            .read_memory::<u32>(pp_items + i * 4)
            .unwrap_or(0);
        if p_item != 0 {
            occupied[i] = true;
        }
    }

    // Look for a contiguous (width x height) rectangle of free cells
    if width > grid_x || height > grid_y {
        return false;
    }

    for y in 0..=(grid_y - height) {
        for x in 0..=(grid_x - width) {
            let mut fits = true;
            'check: for dy in 0..height {
                for dx in 0..width {
                    let idx = (y + dy) * grid_x + (x + dx);
                    if occupied[idx] {
                        fits = false;
                        break 'check;
                    }
                }
            }
            if fits {
                return true;
            }
        }
    }

    false
}

/// Checks whether the Horadric Cube has space for an item of size (width, height).
pub fn check_cube_has_space(ctx: &D2Context, width: usize, height: usize) -> ContainerSpace {
    let p_player = match ctx
        .process
        .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
    {
        Ok(p) if p != 0 => p as usize,
        _ => return ContainerSpace::ContainerNotFound,
    };

    let p_inv = match ctx.process.read_memory::<u32>(p_player + unit::INVENTORY) {
        Ok(p) if p != 0 => p as usize,
        _ => return ContainerSpace::ContainerNotFound,
    };

    let mut p_item = match ctx
        .process
        .read_memory::<u32>(p_inv + inventory::FIRST_ITEM)
    {
        Ok(p) => p,
        Err(_) => return ContainerSpace::ContainerNotFound,
    };

    let mut has_cube = false;
    // In Median XL, Horadric Cube is 10x10.
    const CUBE_W: usize = 10;
    const CUBE_H: usize = 10;
    let mut occupied = vec![false; CUBE_W * CUBE_H];

    for _ in 0..256 {
        if p_item == 0 {
            break;
        }

        let p_unit_data = match ctx
            .process
            .read_memory::<u32>(p_item as usize + unit::UNIT_DATA)
        {
            Ok(p) if p != 0 => p as usize,
            _ => break,
        };

        let inv_page = ctx
            .process
            .read_memory::<u8>(p_unit_data + item_data::INV_PAGE)
            .unwrap_or(255);
        let class_id = ctx
            .process
            .read_memory::<u32>(p_item as usize + unit::CLASS)
            .unwrap_or(0);

        if is_horadric_cube(ctx, class_id) {
            has_cube = true;
        } else if inv_page == 3 {
            // Page 3 = D2ItemInvPage::Cube
            let (item_w, item_h) = read_item_dimensions(ctx, class_id);
            // static path x & y or item_data position
            let p_path = ctx
                .process
                .read_memory::<u32>(p_item as usize + unit::PATH)
                .unwrap_or(0) as usize;
            let ix = if p_path != 0 {
                ctx.process
                    .read_memory::<u32>(p_path + item_path::SUB_X)
                    .unwrap_or(0) as usize
            } else {
                0
            };
            let iy = if p_path != 0 {
                ctx.process
                    .read_memory::<u32>(p_path + item_path::SUB_Y)
                    .unwrap_or(0) as usize
            } else {
                0
            };

            for dy in 0..item_h {
                for dx in 0..item_w {
                    let cx = (ix + dx).min(CUBE_W - 1);
                    let cy = (iy + dy).min(CUBE_H - 1);
                    occupied[cy * CUBE_W + cx] = true;
                }
            }
        }

        p_item = match ctx
            .process
            .read_memory::<u32>(p_unit_data + item_data::NEXT_ITEM)
        {
            Ok(p) => p,
            _ => break,
        };
    }

    if !has_cube {
        return ContainerSpace::ContainerNotFound;
    }

    if width > CUBE_W || height > CUBE_H {
        return ContainerSpace::Full;
    }

    for y in 0..=(CUBE_H - height) {
        for x in 0..=(CUBE_W - width) {
            let mut fits = true;
            'check: for dy in 0..height {
                for dx in 0..width {
                    if occupied[(y + dy) * CUBE_W + (x + dx)] {
                        fits = false;
                        break 'check;
                    }
                }
            }
            if fits {
                return ContainerSpace::Available;
            }
        }
    }

    ContainerSpace::Full
}
