//! Auto Pickup Engine for D2MXLUtils-plus.
//!
//! Provides high-performance, responsive native packet-based auto pickup (packets 0x16 and 0x2A).
//! Fully customizable with cleaned HackMap-style TOML rule format.

pub mod capacity;
pub mod packet;
pub mod rule;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "windows")]
use crate::hotkeys::chord_is_pressed_d2_only;
#[cfg(target_os = "linux")]
use crate::hotkeys::chord_is_pressed_d2_only_linux;
use crate::hotkeys::HotkeyConfig;
use crate::injection::D2Injector;
use crate::logger::{error as log_error, info as log_info};
use crate::notifier::ItemDropEvent;
use crate::offsets::{d2client, inventory, item_data, unit};
use crate::process::D2Context;
pub use rule::{parse_pickup_rules, CompiledPickupRule, PickupAction};

pub const DEFAULT_PICKUP_RULES: &str = r#"# ==============================================================================
# D2MXLUtils-plus 自动拾取规则模板 (基于 HackMap 规则清洗移植)
# 规则说明：
# 1. 匹配机制：自底向上（倒序遍历匹配，写在越靠后的规则优先级越高）。
# 2. 拾取行为 (pickup)：
#    pickup = 0 -> 不捡（用于特定高价值物品防误捡保护，需手动拾取）
#    pickup = 1 -> 直接入背包 (Inventory)
#    pickup = 2 -> 优先放入赫拉迪克方块 (Cube，方块满自动降级入包)
#    pickup = 3 -> 自动补入腰带药水栏 (AutoBelt，腰带满静默跳过)
# 3. 匹配属性：
#    match_name = 1 -> 匹配物品名称 (prop/regex)
#    match_name = 0 -> 匹配装备词条属性 (regex)
#    quality -> 品质 (normal / superior / magic / rare / set / unique)
#    id -> 物品类型 Class ID
#    socks -> 孔数，eth -> 无形
# ==============================================================================

# --- 宝石与通用圣坛 ---
{ match_name = 1, prop = "完美的", pickup = 0 },
{ match_name = 1, prop = "圣坛", pickup = 1 },

# --- 绿色套装与暗金装备 ---
{ quality = "set", pickup = 2 },
{ quality = "unique", pickup = 2 },
{ quality = "unique", prop = "[神圣]", pickup = 2 },

# --- 天使级装备 (极品属性判定与普通词缀排除) ---
{ quality = "superior", eth = 1, pickup = 0 },
{ quality = "rare", match_name = 1, regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", pickup = 0 },
{ quality = "magic", match_name = 1, regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", pickup = 0 },
{ id = 676, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+1[7-9][0-9]\\s*(?:最小伤害|(?:to\\s+)?Minimum\\s+Damage)\\b)", pickup = 0 },
{ id = 678, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+1[7-9][0-9]\\s*(?:最小伤害|(?:to\\s+)?Minimum\\s+Damage)\\b)", pickup = 0 },
{ id = 682, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+1[7-9][0-9]\\s*(?:最小伤害|(?:to\\s+)?Minimum\\s+Damage)\\b)", pickup = 0 },
{ id = 684, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+1[7-9][0-9]\\s*(?:最小伤害|(?:to\\s+)?Minimum\\s+Damage)\\b)", pickup = 0 },
{ id = 685, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+1[7-9][0-9]\\s*(?:最小伤害|(?:to\\s+)?Minimum\\s+Damage)\\b)", pickup = 0 },
{ id = 676, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:最小伤害|最大伤害|(?:to\\s+)?(?:Minimum|Maximum)\\s+Damage)\\b|(?:^|[/\\r\\n])\\s*\\+?[0-9]+\\s*-\\s*(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:伤害|(?:to\\s+)?Damage)\\b)", pickup = 2 },
{ id = 678, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:最小伤害|最大伤害|(?:to\\s+)?(?:Minimum|Maximum)\\s+Damage)\\b|(?:^|[/\\r\\n])\\s*\\+?[0-9]+\\s*-\\s*(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:伤害|(?:to\\s+)?Damage)\\b)", pickup = 2 },
{ id = 682, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:最小伤害|最大伤害|(?:to\\s+)?(?:Minimum|Maximum)\\s+Damage)\\b|(?:^|[/\\r\\n])\\s*\\+?[0-9]+\\s*-\\s*(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:伤害|(?:to\\s+)?Damage)\\b)", pickup = 2 },
{ id = 684, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:最小伤害|最大伤害|(?:to\\s+)?(?:Minimum|Maximum)\\s+Damage)\\b|(?:^|[/\\r\\n])\\s*\\+?[0-9]+\\s*-\\s*(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:伤害|(?:to\\s+)?Damage)\\b)", pickup = 2 },
{ id = 685, name_regex = "(?i)(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:最小伤害|最大伤害|(?:to\\s+)?(?:Minimum|Maximum)\\s+Damage)\\b|(?:^|[/\\r\\n])\\s*\\+?[0-9]+\\s*-\\s*(?:[2-9][0-9]{2}|[1-9][0-9]{3,})\\s*(?:伤害|(?:to\\s+)?Damage)\\b)", pickup = 2 },
{ name_regex = "(?is)(?:(?:轻型装甲|\\bLight\\s+Plate(?:\\s+Armor)?\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻型装甲|\\bLight\\s+Plate(?:\\s+Armor)?\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:(?:^|[/\\r\\n])\\s*\\+?(?:(?:[7-9][0-9]|[1-9][0-9]{2,})\\s*|(?:[2-9][0-9]|[1-9][0-9]{2,})\\s*%\\s*)(?:力量|敏捷|精力|体力|(?:to\\s+)?(?:Strength|Dexterity|Energy|Vitality))\\b|(?:^|[/\\r\\n])\\s*\\+?(?:[4-9][0-9]|[1-9][0-9]{2,})\\s*(?:所有属性|(?:to\\s+)?All\\s+(?:Attributes|Stats))\\b|(?:^|[/\\r\\n])\\s*\\+?(?:1[5-9]|[2-9][0-9]|[1-9][0-9]{2,})\\s*%\\s*(?:所有属性|(?:to\\s+)?All\\s+(?:Attributes|Stats))\\b)", pickup = 2 },
{ name_regex = "(?is)(?:(?:高级头盔|全盔|\\bFull\\s+Helm\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:高级头盔|全盔|\\bFull\\s+Helm\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[4-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 0 },
{ name_regex = "(?is)(?:(?:轻型装甲|\\bLight\\s+Plate(?:\\s+Armor)?\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻型装甲|\\bLight\\s+Plate(?:\\s+Armor)?\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[4-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:轻盾|\\bKite\\s+Shield\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻盾|\\bKite\\s+Shield\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[4-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:轻型金属靴|轻型铁靴|\\bLight\\s+Plated\\s+Boots\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻型金属靴|轻型铁靴|\\bLight\\s+Plated\\s+Boots\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[2-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:轻型铁手套|\\bLight\\s+Gauntlets\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻型铁手套|\\bLight\\s+Gauntlets\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[2-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:轻扣带|腰带|\\b(?:Light\\s+)?Belt\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:轻扣带|腰带|\\b(?:Light\\s+)?Belt\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[2-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:权杖|\\bScepter\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:权杖|\\bScepter\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[5-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:战斗法杖|长棍|\\b(?:Battle\\s+Staff|Long\\s+Staff)\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:战斗法杖|长棍|\\b(?:Battle\\s+Staff|Long\\s+Staff)\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:1[0-9]|[2-9][0-9]|[1-9][0-9]{2,})\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 2 },
{ name_regex = "(?is)(?:(?:三叉戟|\\bTrident\\b).*(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\))|(?:\\[天使级\\]|\\[Angelic\\]|\\(Angelic\\)).*(?:三叉戟|\\bTrident\\b))", quality = "rare", match_name = 0, regex = "(?is)(?:^|[/\\r\\n])\\s*\\+(?:[8-9]|[1-9][0-9]+)\\s*(?:所有技能|(?:to\\s+)?All\\s+Skills)\\b", pickup = 0 },

# --- 特定 Class ID 杂项与符文 ---
{ id = 686, pickup = 1 },
{ id = 1162, pickup = 1 },

# --- 常用合成、重塑与任务材料 (入包) ---
{ match_name = 1, regex = "神秘染料|任务物品|重塑材料", pickup = 1 },

# --- 核心材料与通货 (优先入方块) ---
{ match_name = 1, regex = "萃取物|洗点水|强效好运油|幻梦碎片|星象图|赫拉迪姆方块|碎片|奥术水晶|奥术簇群|奖杯|腐化的宝箱|元素符文", pickup = 2 },
{ match_name = 1, regex = "emblem", pickup = 1 },
{ match_name = 1, regex = "往生之翼|樱桃|物品设计图|腐化水晶|腐化碎片|卷轴|大颗|簇群|VESSEL|之书|穿越器|Horadric", pickup = 2 },
{ match_name = 1, regex = "秘境石|枯竭的秘境石", pickup = 1 },
{ match_name = 1, regex = "精华|原初符文|天堂之魂|胜利之徽", pickup = 1 },
{ match_name = 1, prop = "遗物", pickup = 2 },
{ match_name = 1, prop = "神秘球体", pickup = 2 },
{ match_name = 1, prop = "玄秘塑像", pickup = 2 },
{ match_name = 1, prop = "学习印章", pickup = 2 },

# --- 四属性回环 (直接入包) ---
{ regex = "回环.*力量", pickup = 1 },
{ regex = "回环.*敏捷", pickup = 1 },
{ regex = "回环.*精力", pickup = 1 },
{ regex = "回环.*体力", pickup = 1 },

# --- 符文类配置 ---
{ quality = "normal", match_name = 1, prop = "高级符文", pickup = 2 },
{ quality = "normal", match_name = 1, prop = "强化符文", pickup = 2 },
# 顶级大符文保护：设为0禁止自动拾取，必须手动点击拾取防意外
{ quality = "normal", match_name = 1, regex = "Ber|Jah|Cham|Zod", pickup = 0 },

# --- 药水类配置 ---
# 活力药水直接入包
{ match_name = 1, quality = "normal", regex = "全效活力药水", pickup = 1 },
# 生命/法力药水优先自动补入腰带 (AutoBelt)
{ match_name = 1, quality = "normal", prop = "特效生命药水", pickup = 3 },
{ match_name = 1, quality = "normal", regex = "(生命|治疗|法力|魔法).*(药水|药剂)|(药水|药剂).*(生命|治疗|法力|魔法)", pickup = 3 },
{ match_name = 1, quality = "normal", regex = "全效活力药水|活力药水", pickup = 1 },

# --- 特殊基底与杂项 ---
{ base_code = 0x20786F62, pickup = 1 }, # 赫拉迪克方块基底

# 特定剧情/任务掉落物品排除 (不捡)
{ match_name = 1, prop = "萨卡兰姆的耳朵", pickup = 0 },
{ match_name = 1, prop = "霍拉松的", pickup = 0 },
{ match_name = 1, prop = "阿卡拉特之视界", pickup = 0 },
{ match_name = 1, prop = "谎言之书", pickup = 0 },
{ match_name = 1, prop = "龙爪", pickup = 0 },
{ match_name = 1, prop = "愚者金币", pickup = 0 },

# 特定高阶掉落材料 (入方块)
{ match_name = 1, prop = "剧毒蜇针", pickup = 2 },
{ match_name = 1, prop = "冷核聚变简图", pickup = 2 },
{ match_name = 1, prop = "Trance Herb", pickup = 2 },
{ match_name = 1, prop = "骨堆", pickup = 2 },
"#;

pub struct AutoPickupState {
    pub enabled: AtomicBool,
    pub pickup_distance: Mutex<f32>,
    pub rules: RwLock<Vec<CompiledPickupRule>>,
    last_action: Mutex<Instant>,
    in_flight: Mutex<HashMap<u32, Instant>>,
    pending_cube_item: Mutex<Option<(u32, u32, Instant)>>,
}

impl AutoPickupState {
    pub fn new() -> Self {
        let compiled = parse_pickup_rules(DEFAULT_PICKUP_RULES).unwrap_or_default();
        Self {
            enabled: AtomicBool::new(false),
            pickup_distance: Mutex::new(5.0),
            rules: RwLock::new(compiled),
            last_action: Mutex::new(Instant::now() - Duration::from_secs(5)),
            in_flight: Mutex::new(HashMap::new()),
            pending_cube_item: Mutex::new(None),
        }
    }

    pub fn set_rules(&self, rules_text: &str) -> Result<(), String> {
        let compiled = parse_pickup_rules(rules_text)?;
        let count = compiled.len();
        if let Ok(mut lock) = self.rules.write() {
            *lock = compiled;
        }
        log_info(&format!("AutoPickup rules updated: {} rules loaded", count));
        Ok(())
    }

    pub fn tick(
        &self,
        ctx: &D2Context,
        injector: &Mutex<D2Injector>,
        recent_events: &RwLock<HashMap<u32, ItemDropEvent>>,
    ) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        // 1. Two-phase transition for Horadric Cube pickup:
        // When picking to cube, 0x16 put it on cursor. Now send 0x2A to insert into Cube.
        let pending = {
            let guard = self.pending_cube_item.lock().unwrap();
            *guard
        };

        if let Some((item_uid, cube_uid, start)) = pending {
            let p_player = match ctx
                .process
                .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
            {
                Ok(p) if p != 0 => p as usize,
                _ => return,
            };

            let p_inv = match ctx.process.read_memory::<u32>(p_player + unit::INVENTORY) {
                Ok(p) if p != 0 => p as usize,
                _ => return,
            };

            let cursor_item = ctx
                .process
                .read_memory::<u32>(p_inv + inventory::CURSOR_ITEM)
                .unwrap_or(0);

            if cursor_item != 0 {
                // Item is now on the cursor! Immediately dispatch packet 0x2A
                let cube_packet = packet::build_item_to_cube_packet(item_uid, cube_uid);
                if let Ok(inj) = injector.lock() {
                    match inj.send_packet(&ctx.process, &cube_packet) {
                        Ok(_) => {
                            log_info(&format!(
                                "AutoPickup: moved cursor item 0x{:X} into cube 0x{:X}",
                                item_uid, cube_uid
                            ));
                        }
                        Err(e) => {
                            log_error(&format!("AutoPickup: packet 0x2A failed: {}", e));
                        }
                    }
                }
                *self.pending_cube_item.lock().unwrap() = None;
                if let Ok(mut last) = self.last_action.lock() {
                    *last = Instant::now();
                }
                return;
            } else if start.elapsed() > Duration::from_millis(1500) {
                // Timeout waiting for item on cursor
                *self.pending_cube_item.lock().unwrap() = None;
            } else {
                // Still waiting for server to place item on cursor
                return;
            }
        }

        // 2. Throttle actions (at least 150ms between pickups)
        {
            let last = self.last_action.lock().unwrap();
            if last.elapsed() < Duration::from_millis(150) {
                return;
            }
        }

        // 3. Clean up expired in-flight entries
        {
            let now = Instant::now();
            if let Ok(mut map) = self.in_flight.lock() {
                map.retain(|_, exp| *exp > now);
            }
        }

        // 4. Cursor safety check: if player is already holding an item, do not pick up
        let p_player = match ctx
            .process
            .read_memory::<u32>(ctx.d2_client + d2client::PLAYER_UNIT)
        {
            Ok(p) if p != 0 => p as usize,
            _ => return,
        };

        let p_inv = match ctx.process.read_memory::<u32>(p_player + unit::INVENTORY) {
            Ok(p) if p != 0 => p as usize,
            _ => return,
        };

        if let Ok(cursor_item) = ctx
            .process
            .read_memory::<u32>(p_inv + inventory::CURSOR_ITEM)
        {
            if cursor_item != 0 {
                return;
            }
        }

        // 5. Read player position
        let (player_sx, player_sy) = match crate::map_markers::manager::read_player_subtile(ctx) {
            Some(pos) => pos,
            None => return,
        };

        // 6. Read ground items in rooms near player (depth 2 BFS)
        let positions = match crate::map_markers::manager::bfs_item_positions(ctx, 2) {
            Ok(p) => p,
            Err(_) => return,
        };

        if positions.is_empty() {
            return;
        }

        let max_distance = *self.pickup_distance.lock().unwrap();
        let rules_lock = self.rules.read().unwrap();
        if rules_lock.is_empty() {
            return;
        }

        let events_cache = recent_events.read().unwrap();
        let in_flight_map = self.in_flight.lock().unwrap();

        // 7. Find matching candidates and sort by distance
        let mut candidates = Vec::new();

        for (p_unit, item_sx, item_sy) in positions {
            let unit_id = match ctx
                .process
                .read_memory::<u32>(p_unit as usize + unit::UNIT_ID)
            {
                Ok(uid) if uid != 0 => uid,
                _ => continue,
            };

            if in_flight_map.contains_key(&unit_id) {
                continue;
            }

            let dx = player_sx - item_sx;
            let dy = player_sy - item_sy;
            let dist_yards = (((dx * dx + dy * dy) as f32).sqrt()) / 5.0;

            if dist_yards > max_distance {
                continue;
            }

            candidates.push((p_unit, unit_id, dist_yards));
        }

        drop(in_flight_map);

        if candidates.is_empty() {
            return;
        }

        // Sort closest first
        candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        for (p_unit, unit_id, dist) in candidates {
            let class_id = ctx
                .process
                .read_memory::<u32>(p_unit as usize + unit::CLASS)
                .unwrap_or(0);
            let base_code = capacity::read_item_code(ctx, class_id);

            let p_unit_data = match ctx
                .process
                .read_memory::<u32>(p_unit as usize + unit::UNIT_DATA)
            {
                Ok(p) if p != 0 => p as usize,
                _ => continue,
            };

            let quality = ctx
                .process
                .read_memory::<u32>(p_unit_data + item_data::QUALITY)
                .unwrap_or(0);
            let flags = ctx
                .process
                .read_memory::<u32>(p_unit_data + item_data::FLAGS)
                .unwrap_or(0);
            let is_eth = (flags & crate::offsets::item_flags::ETHEREAL) != 0;

            // Get name and properties from DropScanner's recent_events if available
            let (name, props) = if let Some(ev) = events_cache.get(&unit_id) {
                (ev.name.as_str(), ev.stats.as_str())
            } else {
                ("", "")
            };

            // Socket count
            let sockets = if let Some(ev) = events_cache.get(&unit_id) {
                ev.sockets as usize
            } else {
                0
            };

            // Reverse match (last match wins)
            let matched_action = rules_lock.iter().rev().find_map(|rule| {
                if rule.matches(class_id, base_code, quality, is_eth, sockets, name, props) {
                    Some(rule.pickup)
                } else {
                    None
                }
            });

            let action = match matched_action {
                Some(a) => a,
                None => continue,
            };

            if action == PickupAction::None {
                continue;
            }

            let (width, height) = capacity::read_item_dimensions(ctx, class_id);

            match action {
                PickupAction::None => continue,

                PickupAction::AutoBelt => {
                    // Check if belt has room
                    if !capacity::check_belt_has_empty_slot(ctx) {
                        continue;
                    }

                    // Dispatch 0x16 with bCursor = 0 (native game logic fills belt)
                    let packet = packet::build_pickup_packet(unit_id, false);
                    if let Ok(inj) = injector.lock() {
                        if inj.send_packet(&ctx.process, &packet).is_ok() {
                            log_info(&format!(
                                "AutoPickup: dispatched AutoBelt pickup for unit 0x{:X} (dist: {:.1}y)",
                                unit_id, dist
                            ));
                            self.register_in_flight(unit_id);
                            return;
                        }
                    }
                }

                PickupAction::Cube => {
                    let cube_uid = capacity::find_horadric_cube(ctx);
                    let can_cube = cube_uid.is_some()
                        && capacity::check_cube_has_space(ctx, width, height)
                            == capacity::ContainerSpace::Available;

                    if can_cube {
                        // Phase 1: pick up to cursor
                        let packet = packet::build_pickup_packet(unit_id, true);
                        if let Ok(inj) = injector.lock() {
                            if inj.send_packet(&ctx.process, &packet).is_ok() {
                                log_info(&format!(
                                    "AutoPickup: dispatched Cube pickup phase-1 for unit 0x{:X}",
                                    unit_id
                                ));
                                self.register_in_flight(unit_id);
                                *self.pending_cube_item.lock().unwrap() =
                                    Some((unit_id, cube_uid.unwrap(), Instant::now()));
                                return;
                            }
                        }
                    } else if capacity::check_inventory_has_space(ctx, width, height) {
                        // Fallback to regular inventory
                        let packet = packet::build_pickup_packet(unit_id, false);
                        if let Ok(inj) = injector.lock() {
                            if inj.send_packet(&ctx.process, &packet).is_ok() {
                                log_info(&format!(
                                    "AutoPickup: cube unavailable/full, picked to inventory 0x{:X}",
                                    unit_id
                                ));
                                self.register_in_flight(unit_id);
                                return;
                            }
                        }
                    }
                }

                PickupAction::Inventory => {
                    // Check inventory space
                    if !capacity::check_inventory_has_space(ctx, width, height) {
                        continue;
                    }

                    let packet = packet::build_pickup_packet(unit_id, false);
                    if let Ok(inj) = injector.lock() {
                        if inj.send_packet(&ctx.process, &packet).is_ok() {
                            log_info(&format!(
                                "AutoPickup: dispatched Inventory pickup for unit 0x{:X} (dist: {:.1}y)",
                                unit_id, dist
                            ));
                            self.register_in_flight(unit_id);
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.in_flight.lock() {
            map.clear();
        }
        if let Ok(mut pending) = self.pending_cube_item.lock() {
            *pending = None;
        }
    }

    fn register_in_flight(&self, unit_id: u32) {
        if let Ok(mut map) = self.in_flight.lock() {
            map.insert(unit_id, Instant::now() + Duration::from_secs(3));
        }
        if let Ok(mut last) = self.last_action.lock() {
            *last = Instant::now();
        }
    }
}

pub fn default_auto_pickup_hotkey() -> HotkeyConfig {
    HotkeyConfig {
        key_code: 0x21, // VK_PRIOR (PageUp)
        modifiers: 0,
        display: "PageUp".to_string(),
    }
}

pub struct AutoPickupHotkeyState {
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
}

impl AutoPickupHotkeyState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            current_hotkey: Arc::new(Mutex::new(Some(default_auto_pickup_hotkey()))),
        }
    }

    pub fn update(&self, hotkey: Option<HotkeyConfig>) {
        if let Ok(mut current) = self.current_hotkey.lock() {
            *current = hotkey;
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn start(&self, app_handle: AppHandle, hotkey: Option<HotkeyConfig>) {
        if self.is_running.load(Ordering::SeqCst) {
            self.stop();
            thread::sleep(Duration::from_millis(60));
        }

        if let Ok(mut current) = self.current_hotkey.lock() {
            *current = hotkey;
        }

        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let current_hotkey = self.current_hotkey.clone();

        #[cfg(target_os = "windows")]
        thread::spawn(move || {
            auto_pickup_hotkey_thread_windows(is_running, current_hotkey, app_handle);
        });

        #[cfg(target_os = "linux")]
        thread::spawn(move || {
            auto_pickup_hotkey_thread_linux(is_running, current_hotkey, app_handle);
        });

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = (is_running, current_hotkey, app_handle);
        }
    }
}

#[cfg(target_os = "windows")]
fn auto_pickup_hotkey_thread_windows(
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    app_handle: AppHandle,
) {
    let mut was_active = false;
    while is_running.load(Ordering::SeqCst) {
        let hk_opt = match current_hotkey.lock() {
            Ok(g) => g.clone(),
            Err(_) => break,
        };

        let active = if let Some(ref hk) = hk_opt {
            chord_is_pressed_d2_only(hk)
        } else {
            false
        };

        if active && !was_active {
            if let Some(state) = app_handle.try_state::<AutoPickupState>() {
                let prev = state.enabled.load(Ordering::Relaxed);
                let new_val = !prev;
                state.enabled.store(new_val, Ordering::Relaxed);
                log_info(&format!("AutoPickup toggled via hotkey: {}", new_val));
                let _ = app_handle.emit("auto-pickup-toggled", new_val);
            }
        }
        was_active = active;
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(target_os = "linux")]
fn auto_pickup_hotkey_thread_linux(
    is_running: Arc<AtomicBool>,
    current_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    app_handle: AppHandle,
) {
    let mut was_active = false;
    while is_running.load(Ordering::SeqCst) {
        let hk_opt = match current_hotkey.lock() {
            Ok(g) => g.clone(),
            Err(_) => break,
        };

        let active = if let Some(ref hk) = hk_opt {
            chord_is_pressed_d2_only_linux(hk)
        } else {
            false
        };

        if active && !was_active {
            if let Some(state) = app_handle.try_state::<AutoPickupState>() {
                let prev = state.enabled.load(Ordering::Relaxed);
                let new_val = !prev;
                state.enabled.store(new_val, Ordering::Relaxed);
                log_info(&format!("AutoPickup toggled via hotkey: {}", new_val));
                let _ = app_handle.emit("auto-pickup-toggled", new_val);
            }
        }
        was_active = active;
        thread::sleep(Duration::from_millis(50));
    }
}

/// Tauri command to toggle auto pickup
#[tauri::command]
pub fn toggle_auto_pickup(
    state: tauri::State<'_, AutoPickupState>,
    enabled: bool,
) -> Result<(), String> {
    state.enabled.store(enabled, Ordering::Relaxed);
    log_info(&format!(
        "AutoPickup toggled via command: enabled = {}",
        enabled
    ));
    Ok(())
}

/// Tauri command to update auto pickup rules dynamically
#[tauri::command]
pub fn update_auto_pickup_rules(
    state: tauri::State<'_, AutoPickupState>,
    rules: String,
) -> Result<(), String> {
    state.set_rules(&rules)
}

/// Tauri command to update auto pickup hotkey dynamically
#[tauri::command]
pub fn update_auto_pickup_hotkey(
    state: tauri::State<'_, AutoPickupHotkeyState>,
    hotkey: Option<HotkeyConfig>,
) -> Result<(), String> {
    state.update(hotkey);
    Ok(())
}
