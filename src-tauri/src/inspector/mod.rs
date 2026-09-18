//! Read-only Inspector Module
//! Safely samples hovered monster resistances/stats and hovered item details without code injection.

use serde::Serialize;

use crate::injection::D2Injector;
use crate::offsets;
use crate::process::D2Context;
use crate::unit_stats_reader::UnitStatsReader;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MonsterInspectData {
    pub name: String,
    pub class_id: u32,
    pub unit_id: u32,
    pub hp_percent: u32,
    pub cur_hp: u32,
    pub max_hp: u32,
    pub dr: i32, // Physical
    pub mr: i32, // Magic
    pub fr: i32, // Fire
    pub lr: i32, // Lightning
    pub cr: i32, // Cold
    pub pr: i32, // Poison
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ItemInspectData {
    pub name: String,
    pub class_id: u32,
    pub unit_id: u32,
    pub quality: String,
    pub sockets: u32,
    pub is_ethereal: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum InspectPayload {
    Monster(MonsterInspectData),
    Item(ItemInspectData),
    None,
}

pub fn sample_hovered_monster(
    ctx: &D2Context,
    injector: &D2Injector,
) -> Option<MonsterInspectData> {
    let p_unit = ctx
        .process
        .read_memory::<u32>(ctx.d2_client + offsets::d2client::SELECTED_UNIT)
        .ok()?;
    if p_unit == 0 {
        return None;
    }

    let unit_type = ctx.process.read_memory::<u32>(p_unit as usize).ok()?;
    if unit_type != offsets::unit_type::MONSTER {
        return None;
    }

    let class_id = ctx
        .process
        .read_memory::<u32>(p_unit as usize + offsets::unit::CLASS)
        .ok()?;
    let unit_id = ctx
        .process
        .read_memory::<u32>(p_unit as usize + offsets::unit::UNIT_ID)
        .ok()?;

    // Read monster name from MonStatsTxt if possible
    let mut name = String::new();
    if let Ok(p_unit_data) = ctx
        .process
        .read_memory::<u32>(p_unit as usize + offsets::unit::UNIT_DATA)
    {
        if p_unit_data != 0 {
            if let Ok(p_monstats) = ctx.process.read_memory::<u32>(p_unit_data as usize) {
                if p_monstats != 0 {
                    if let Ok(w_name_str) =
                        ctx.process.read_memory::<u16>(p_monstats as usize + 0x06)
                    {
                        if w_name_str != 0 {
                            if let Ok(str_val) = injector.get_string(&ctx.process, w_name_str, 64) {
                                name = str_val.trim().to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    if name.is_empty() {
        name = format!("Monster #{class_id}");
    }

    // Read stats
    let reader = UnitStatsReader::new(&ctx.process, ctx.d2_common, p_unit);
    let stats = reader
        .read_bulk(&[6, 7, 36, 37, 39, 41, 43, 45], 0)
        .unwrap_or_default();

    let cur_hp = (stats.get(&6).copied().unwrap_or(0) >> 8).max(0) as u32;
    let max_hp = (stats.get(&7).copied().unwrap_or(0) >> 8).max(1) as u32;
    let hp_percent = ((cur_hp as u64 * 100) / max_hp as u64).min(100) as u32;

    Some(MonsterInspectData {
        name,
        class_id,
        unit_id,
        hp_percent,
        cur_hp,
        max_hp,
        dr: stats.get(&36).copied().unwrap_or(0),
        mr: stats.get(&37).copied().unwrap_or(0),
        fr: stats.get(&39).copied().unwrap_or(0),
        lr: stats.get(&41).copied().unwrap_or(0),
        cr: stats.get(&43).copied().unwrap_or(0),
        pr: stats.get(&45).copied().unwrap_or(0),
    })
}
