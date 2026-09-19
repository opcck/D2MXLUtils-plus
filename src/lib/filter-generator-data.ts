/**
 * 掉落过滤规则生成器数据字典与元数据定义
 * 数据源依据：严格对应 patchstring.txt、string.txt、expansionstring.txt 官方中文汉化表。
 * 没有对应中文的保持英文原生标识，不进行额外意译。
 */

export interface ItemCategoryOption {
  label: string;
  pattern: string;
  group?: string;
}

export interface StatOption {
  id: string;
  label: string;
  token: string;
  defaultRegex: string;
  valuePrefix?: string;
  valueSuffix?: string;
  exampleMin?: number;
}

export interface PresetOption {
  name: string;
  desc: string;
  rule: {
    namePattern?: string;
    quality?: string;
    tier?: string;
    sockets?: string;
    minClvl?: number | null;
    maxClvl?: number | null;
    minIlvl?: number | null;
    maxIlvl?: number | null;
    charClass?: string;
    eth?: boolean;
    stats?: { statId: string; minValue?: number | null; customRegex?: string }[];
    color?: string;
    action?: string;
    sound?: string;
    notify?: boolean;
    stat?: boolean;
    map?: boolean;
  };
}

/** 物品大类与中英文对照预设 */
export const ITEM_CATEGORIES: ItemCategoryOption[] = [
  // --- 装备-首饰 ---
  { group: '装备-首饰 Jewelry', label: '戒指 Ring', pattern: 'Ring$' },
  { group: '装备-首饰 Jewelry', label: '护符 Amulet', pattern: 'Amulet$' },
  { group: '装备-首饰 Jewelry', label: '珠宝 Jewel', pattern: 'Jewel' },

  // --- 装备-防具 ---
  {
    group: '装备-盔甲 Armor',
    label: '装备-盔甲 Armor (全防具)',
    pattern:
      'Quilted Armor|Leather Armor|Hard Leather Armor|Studded Leather|Ring Mail|Scale Mail|Chain Mail|Breast Plate|Splint Mail|Plate Mail|Field Plate|Light Plate|Gothic Plate|Full Plate Mail|Ancient Armor',
  },
  {
    group: '装备-盔甲 Armor',
    label: '头盔 Helm',
    pattern: 'Cap|Skull Cap|Helm|Full Helm|Great Helm|Crown|Mask|Bone Helm',
  },
  { group: '装备-盔甲 Armor', label: '头饰 Circlet', pattern: 'Circlet|Coronet|Tiara|Diadem' },
  {
    group: '装备-盔甲 Armor',
    label: '装备-盾牌 Shield',
    pattern:
      'Buckler|Small Shield|Large Shield|Kite Shield|Tower Shield|Gothic Shield|Bone Shield|Spiked Shield|Luna|Hyperion|Monarch|Aegis|Ward|Troll Nest',
  },
  {
    group: '装备-盔甲 Armor',
    label: '腰带 Belt',
    pattern: 'Sash|Light Belt|Belt|Heavy Belt|Plated Belt',
  },
  {
    group: '装备-盔甲 Armor',
    label: '鞋子 Boots',
    pattern: 'Boots|Heavy Boots|Chain Boots|Light Plated Boots|Greaves',
  },
  {
    group: '装备-盔甲 Armor',
    label: '手套 Gloves',
    pattern: 'Leather Gloves|Heavy Gloves|Chain Gloves|Light Gauntlets|Gauntlets',
  },

  // --- 装备-武器 ---
  {
    group: '装备-武器 Weapon',
    label: '单手剑 Sword',
    pattern: 'Short Sword|Scimitar|Saber|Falchion|Broad Sword|Long Sword|War Sword',
  },
  {
    group: '装备-武器 Weapon',
    label: '双手剑 Two-Handed Sword',
    pattern: 'Two-Handed Sword|Claymore|Giant Sword|Bastard Sword|Flamberge|Great Sword',
  },
  { group: '装备-武器 Weapon', label: '武器类型 - 水晶剑 Crystal Sword', pattern: 'Crystal Sword' },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 斧 Axe',
    pattern:
      'Hand Axe|Axe$|Double Axe|Military Pick|War Axe|Large Axe|Broad Axe|Battle Axe|Great Axe|Giant Axe',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 钉锤 Mace',
    pattern: 'Club|Spiked Club|Mace$|Morning Star|Flail',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 锤子 Hammer',
    pattern: 'War Hammer|Maul|Great Maul',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 权杖 Scepter',
    pattern: 'Scepter|Grand Scepter|War Scepter',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 标枪 Javelin',
    pattern: 'Javelin|Pilum|Short Spear|Glaive|Throwing Spear',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 矛 Spear',
    pattern: 'Spear|Trident|Brandistock|Spetum|Pike',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 弓 Bow',
    pattern:
      "Short Bow|Hunter's Bow|Long Bow|Composite Bow|Short Battle Bow|Long Battle Bow|Short War Bow|Long War Bow",
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 弩 Crossbow',
    pattern: 'Light Crossbow|Crossbow|Heavy Crossbow|Repeating Crossbow',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 匕首 Dagger',
    pattern: 'Dagger|Dirk|Kriss|Blade$',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 法杖 Staff',
    pattern: 'Short Staff|Long Staff|Gnarled Staff|Battle Staff|War Staff',
  },
  { group: '装备-武器 Weapon', label: '武器类型 - 镰刀 Scythe', pattern: 'Scythe' },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 爪 Claws',
    pattern: 'Katar|Wrist Blade|Hatchet Hands|Cestus|Claws|Blade Talons|Scissors Katar',
  },
  { group: '装备-武器 Weapon', label: '武器类型 - 薙刀 Naginata', pattern: 'Naginata' },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 飞刀 Throwing Knife',
    pattern: 'Throwing Knife|Flying Knife|Balanced Knife',
  },
  {
    group: '装备-武器 Weapon',
    label: '武器类型 - 飞斧 Throwing Axe',
    pattern: 'Throwing Axe|Balanced Axe',
  },
  { group: '装备-武器 Weapon', label: '装备-箭袋 Quiver', pattern: 'Quiver' },

  // --- 专属-职业专属 ---
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-亚马逊 Class Specific Amazon',
    pattern:
      'Stag Bow|Reflex Bow|Maiden Spear|Maiden Pike|Maiden Javelin|Amazon Helm|Amazon Shield',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-刺客 Class Specific Assassin',
    pattern:
      'Katar|Wrist Blade|Hatchet Hands|Cestus|Claws|Blade Talons|Scissors Katar|Assassin Shield',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-野蛮人 Class Specific Barbarian',
    pattern: 'Barbarian Helm|Barbarian Shield',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-德鲁伊 Class Specific Druid',
    pattern:
      'Druid Helm|Pelt|Antlers|Compound Bow|Serpent Bow|Maple Bow|Viper Bow|Recurve Bow|Flamen Staff',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-死灵法师 Class Specific Necromancer',
    pattern:
      'Wand|Yew Wand|Bone Wand|Grim Wand|Necromancer Shield|Marrow Staff|Hexblade|Spirit Edge|Raptor Scythe|Bonesplitter',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-圣骑士 Class Specific Paladin',
    pattern:
      'Paladin Helm|Paladin Shield|Sacred Targe|Sacred Rondache|Kurast Shield|Zakarum Shield|Vortex Shield|Bonebreaker|Goedendag|Angel Star|Hand of God|Holy Lance|Tepoztopilli',
  },
  {
    group: '专属-职业专属 Class Specific',
    label: '专属-法师 Class Specific Sorceress',
    pattern:
      "Eagle Orb|Sacred Globe|Smoked Sphere|Clasped Orb|Jared's Stone|Warp Blade|Sorceress Armor",
  },

  // --- 消耗品与杂项 ---
  {
    group: '消耗品与杂项',
    label: '镶嵌物-高级符文 Great Runes',
    pattern: '^(Ber|Jah|Cham|Zod|Taha|Ghal|Qor|Suhe|Kra|Krys|Auhe|Shaad)$',
  },
  { group: '消耗品与杂项', label: '镶嵌物-符文 Rune (全符文)', pattern: 'Rune$' },
  {
    group: '消耗品与杂项',
    label: '镶嵌物-普通符文 Common Runes (El~Sur)',
    pattern:
      '^(El|Eld|Tir|Nef|Eth|Ith|Tal|Ral|Ort|Thul|Amn|Sol|Shael|Dol|Hel|Io|Lum|Ko|Fal|Lem|Pul|Um|Mal|Ist|Gul|Vex|Ohm|Lo|Sur)$',
  },
  { group: '消耗品与杂项', label: '镶嵌物-元素符文 Elemental Runes', pattern: '^(Pyr|Nox|Yul)$' },
  {
    group: '消耗品与杂项',
    label: '宝石 Gem',
    pattern:
      'Chipped|Flawed|Normal|Flawless|Perfect|Amethyst|Diamond|Emerald|Ruby|Sapphire|Topaz|Skull$',
  },
  {
    group: '消耗品与杂项',
    label: '宝石-完美的宝石 Perfect Gem',
    pattern: 'Perfect (Amethyst|Diamond|Emerald|Ruby|Sapphire|Topaz|Skull)',
  },
  { group: '消耗品与杂项', label: '圣坛 Shrine', pattern: 'Shrine' },
  { group: '消耗品与杂项', label: '神秘球体 Mystic Orb', pattern: 'Mystic Orb' },
  { group: '消耗品与杂项', label: '神秘球体-独特 Mystic Orb Unique', pattern: 'Unique Mystic Orb' },
  { group: '消耗品与杂项', label: '印章 Signet', pattern: 'Signet' },
  { group: '消耗品与杂项', label: '印章-属性印章 Signet Attribute', pattern: 'Signet of Learning' },
  { group: '消耗品与杂项', label: '印章-金钱印章 Signet Gold', pattern: 'Signet of Gold' },
  { group: '消耗品与杂项', label: '回环 Cycle', pattern: 'Cycle' },
  { group: '消耗品与杂项', label: '遗物 Relic', pattern: 'Relic' },
  { group: '消耗品与杂项', label: '水晶类-奥术水晶 Arcane Crystal', pattern: 'Arcane Crystal' },
  { group: '消耗品与杂项', label: '水晶类-奥术碎片 Arcane Shard', pattern: 'Arcane Shard' },
  { group: '消耗品与杂项', label: '秘境石 Riftstone', pattern: 'Riftstone' },
  {
    group: '消耗品与杂项',
    label: '方块重塑材料 Cube Reagent',
    pattern: 'Cube Reagent|Catalyst of|Oil of',
  },
  { group: '消耗品与杂项', label: '奖杯 Trophy', pattern: 'Trophy' },
  { group: '消耗品与杂项', label: '元素精华 Essence', pattern: 'Essence' },
  {
    group: '消耗品与杂项',
    label: '药水 Potion',
    pattern: 'Potion|Elixir|Antidote|Thawing|Stamina',
  },
  { group: '消耗品与杂项', label: '任务物品 Quest Item', pattern: 'Quest Item' },
  { group: '消耗品与杂项', label: '魔方 Cube / 赫拉迪姆方块', pattern: 'Horadric Cube|Cube' },
];

/** 品质选项 (对应 patchstring.txt) */
export const QUALITY_OPTIONS = [
  { value: '', label: '不限' },
  { value: 'unique', label: '暗金 Unique' },
  { value: 'set', label: '套装 Part of a Set' },
  { value: 'rare', label: '黄装 Rare' },
  { value: 'magic', label: '蓝装 Magic' },
  { value: 'craft', label: '手工(橙装) Crafted' },
  { value: 'honor', label: '荣耀 Honorific' },
  { value: 'normal', label: '普通 Normal' },
  { value: 'superior', label: '超强的 High Quality' },
  { value: 'low', label: '低劣的 Low Quality' },
  { value: 'tu', label: 'TU' },
  { value: 'su', label: 'SU (神圣暗金)' },
  { value: 'ssu', label: 'SSU' },
  { value: 'sssu', label: 'SSSU' },
];

/** 品级选项 (对应 patchstring.txt itemcls_) */
export const TIER_OPTIONS = [
  { value: '', label: '不限' },
  { value: 'sacred', label: '品级 [神圣] (Sacred)' },
  { value: 'angelic', label: '品级 [天使级] (Angelic)' },
  { value: 'master', label: '品级 [大师级] (Mastercrafted)' },
  { value: '1', label: '品级 1' },
  { value: '2', label: '品级 2' },
  { value: '3', label: '品级 3' },
  { value: '4', label: '品级 4' },
  { value: '0', label: '品级 0' },
];

/** 镶孔选项 (对应 patchstring.txt) */
export const SOCKET_OPTIONS = [
  { value: '', label: '不限' },
  { value: 'sockets0', label: '无镶孔 (0孔)' },
  { value: 'sockets1', label: '1 镶孔' },
  { value: 'sockets2', label: '2 镶孔' },
  { value: 'sockets3', label: '3 镶孔' },
  { value: 'sockets4', label: '4 镶孔' },
  { value: 'sockets5', label: '5 镶孔' },
  { value: 'sockets6', label: '6 镶孔' },
];

/** 专属职业限定 (对应 patchstring.txt itemcls_ 专属分类) */
export const CLASS_OPTIONS = [
  { value: '', label: '不限' },
  { value: 'amazon', label: '专属-亚马逊 Class Specific Amazon' },
  { value: 'sorceress', label: '专属-法师 Class Specific Sorceress' },
  { value: 'necromancer', label: '专属-死灵法师 Class Specific Necromancer' },
  { value: 'paladin', label: '专属-圣骑士 Class Specific Paladin' },
  { value: 'barbarian', label: '专属-野蛮人 Class Specific Barbarian' },
  { value: 'druid', label: '专属-德鲁伊 Class Specific Druid' },
  { value: 'assassin', label: '专属-刺客 Class Specific Assassin' },
];

/** 颜色选项 */
export const COLOR_OPTIONS = [
  { value: '', label: '默认颜色', hex: '#cccccc' },
  { value: 'gold', label: '黄金 Gold', hex: '#d4af37' },
  { value: 'red', label: '赤红 Red', hex: '#ff4444' },
  { value: 'lime', label: 'lime (亮绿)', hex: '#32cd32' },
  { value: 'blue', label: '蓝色 Blue', hex: '#6688ff' },
  { value: 'white', label: '白色 White', hex: '#ffffff' },
  { value: 'yellow', label: '黄色 Yellow', hex: '#ffff55' },
  { value: 'orange', label: '橙橘 Orange', hex: '#ffaa00' },
  { value: 'pink', label: '粉色 Pink', hex: '#ff69b4' },
  { value: 'grey', label: '幽暗 Grey', hex: '#888888' },
  { value: 'black', label: '夜璃 Black', hex: '#444444' },
  { value: 'purple', label: '紫色 Purple', hex: '#aa55ff' },
  { value: 'green', label: '绿色 Green', hex: '#00aa00' },
];

/** 地面显隐动作 (对应 patchstring.txt) */
export const ACTION_OPTIONS = [
  { value: '', label: '默认 (跟随全局)' },
  { value: 'show', label: '显示 (show)' },
  { value: 'hide', label: '隐藏 (hide)' },
];

/** 音效选项 */
export const SOUND_OPTIONS = [
  { value: '', label: '不播放提示音' },
  { value: 'sound1', label: 'sound1 (槽位 1)' },
  { value: 'sound2', label: 'sound2 (槽位 2)' },
  { value: 'sound3', label: 'sound3 (槽位 3)' },
  { value: 'sound4', label: 'sound4 (槽位 4)' },
  { value: 'sound5', label: 'sound5 (槽位 5)' },
  { value: 'sound6', label: 'sound6 (槽位 6)' },
  { value: 'sound7', label: 'sound7 (槽位 7)' },
  { value: 'sound_none', label: 'sound_none (强制静音)' },
];

/** 常用属性词条列表 (严格匹配 patchstring.txt / string.txt) */
export const STAT_OPTIONS: StatOption[] = [
  {
    id: 'all_skills',
    label: '所有技能 (to All Skills)',
    token: 'to All Skills',
    defaultRegex: 'to All Skills',
    valuePrefix: '\\+?',
    valueSuffix: ' to All Skills',
    exampleMin: 2,
  },
  {
    id: 'str',
    label: '力量 (STR)',
    token: 'to Strength',
    defaultRegex: 'to Strength',
    valuePrefix: '\\+?',
    valueSuffix: ' to Strength',
    exampleMin: 20,
  },
  {
    id: 'dex',
    label: '敏捷 (DEX)',
    token: 'to Dexterity',
    defaultRegex: 'to Dexterity',
    valuePrefix: '\\+?',
    valueSuffix: ' to Dexterity',
    exampleMin: 20,
  },
  {
    id: 'vit',
    label: '体力 (VIT)',
    token: 'to Vitality',
    defaultRegex: 'to Vitality',
    valuePrefix: '\\+?',
    valueSuffix: ' to Vitality',
    exampleMin: 20,
  },
  {
    id: 'eng',
    label: '精力 (ENG)',
    token: 'to Energy',
    defaultRegex: 'to Energy',
    valuePrefix: '\\+?',
    valueSuffix: ' to Energy',
    exampleMin: 20,
  },
  {
    id: 'all_attr',
    label: '所有属性 (All Attributes)',
    token: 'All Attributes',
    defaultRegex: 'All Attributes',
    valuePrefix: '\\+?',
    valueSuffix: ' (to )?All Attributes',
    exampleMin: 15,
  },
  {
    id: 'all_res',
    label: '所有抗性 (All Resistances)',
    token: 'All Resistances',
    defaultRegex: 'All Resistances',
    valuePrefix: '\\+?',
    valueSuffix: '%? All Resistances',
    exampleMin: 20,
  },
  {
    id: 'fire_res',
    label: '火焰抗性 (FR)',
    token: 'Fire Resist',
    defaultRegex: 'Fire Resist',
    valuePrefix: '\\+?',
    valueSuffix: '%? Fire Resist',
    exampleMin: 30,
  },
  {
    id: 'cold_res',
    label: '冰冷抗性 (CR)',
    token: 'Cold Resist',
    defaultRegex: 'Cold Resist',
    valuePrefix: '\\+?',
    valueSuffix: '%? Cold Resist',
    exampleMin: 30,
  },
  {
    id: 'light_res',
    label: '闪电抗性 (LR)',
    token: 'Lightning Resist',
    defaultRegex: 'Lightning Resist',
    valuePrefix: '\\+?',
    valueSuffix: '%? Lightning Resist',
    exampleMin: 30,
  },
  {
    id: 'poison_res',
    label: '毒素抗性 (PR)',
    token: 'Poison Resist',
    defaultRegex: 'Poison Resist',
    valuePrefix: '\\+?',
    valueSuffix: '%? Poison Resist',
    exampleMin: 30,
  },
  {
    id: 'phys_res',
    label: '物理抗性 (Physical Resist)',
    token: 'Physical Resist',
    defaultRegex: 'Physical Resist',
    valuePrefix: '\\+?',
    valueSuffix: '%? Physical Resist',
    exampleMin: 10,
  },
  {
    id: 'ed',
    label: '增强伤害 (ED)',
    token: 'Enhanced Damage',
    defaultRegex: 'Enhanced Damage',
    valuePrefix: '\\+?',
    valueSuffix: '% Enhanced Damage',
    exampleMin: 150,
  },
  {
    id: 'ias',
    label: '攻击速度 (IAS)',
    token: 'Increased Attack Speed',
    defaultRegex: 'Increased Attack Speed',
    valuePrefix: '\\+?',
    valueSuffix: '% Increased Attack Speed',
    exampleMin: 30,
  },
  {
    id: 'fcr',
    label: '施法速度 (FCR)',
    token: 'Faster Cast Rate',
    defaultRegex: 'Faster Cast Rate',
    valuePrefix: '\\+?',
    valueSuffix: '% Faster Cast Rate',
    exampleMin: 30,
  },
  {
    id: 'fhr',
    label: '打击恢复 (FHR)',
    token: 'Faster Hit Recovery',
    defaultRegex: 'Faster Hit Recovery',
    valuePrefix: '\\+?',
    valueSuffix: '% Faster Hit Recovery',
    exampleMin: 30,
  },
  {
    id: 'frw',
    label: '移动速度 (FRW)',
    token: 'Faster Run/Walk',
    defaultRegex: 'Faster Run/Walk',
    valuePrefix: '\\+?',
    valueSuffix: '% Faster Run/Walk',
    exampleMin: 30,
  },
  {
    id: 'cb',
    label: '压碎性打击 (CB)',
    token: 'Chance of Crushing Blow',
    defaultRegex: 'Chance of Crushing Blow',
    valuePrefix: '\\+?',
    valueSuffix: '% Chance of Crushing Blow',
    exampleMin: 15,
  },
  {
    id: 'ds',
    label: '致命攻击 (DS)',
    token: 'Deadly Strike',
    defaultRegex: 'Deadly Strike',
    valuePrefix: '\\+?',
    valueSuffix: '% Deadly Strike',
    exampleMin: 15,
  },
  {
    id: 'ow',
    label: '撕开伤口 (OW)',
    token: 'Chance of Open Wounds',
    defaultRegex: 'Chance of Open Wounds',
    valuePrefix: '\\+?',
    valueSuffix: '% Chance of Open Wounds',
    exampleMin: 15,
  },
  {
    id: 'll',
    label: '偷取生命 (Life Leech)',
    token: 'Life stolen per hit',
    defaultRegex: 'Life stolen per hit',
    valuePrefix: '\\+?',
    valueSuffix: '% Life stolen per hit',
    exampleMin: 8,
  },
  {
    id: 'ml',
    label: '偷取法力 (Mana Leech)',
    token: 'Mana stolen per hit',
    defaultRegex: 'Mana stolen per hit',
    valuePrefix: '\\+?',
    valueSuffix: '% Mana stolen per hit',
    exampleMin: 8,
  },
  {
    id: 'max_life',
    label: '最大生命 (Maximum Life)',
    token: 'Maximum Life',
    defaultRegex: 'Maximum Life',
    valuePrefix: '\\+?',
    valueSuffix: '%? Maximum Life',
    exampleMin: 10,
  },
  {
    id: 'max_mana',
    label: '最大法力 (Maximum Mana)',
    token: 'Maximum Mana',
    defaultRegex: 'Maximum Mana',
    valuePrefix: '\\+?',
    valueSuffix: '%? Maximum Mana',
    exampleMin: 10,
  },
  {
    id: 'spell_focus',
    label: '法术专注 (Spell Focus)',
    token: 'to Spell Focus',
    defaultRegex: 'to Spell Focus',
    valuePrefix: '\\+?',
    valueSuffix: ' to Spell Focus',
    exampleMin: 50,
  },
  {
    id: 'spell_damage',
    label: '法术伤害 (to Spell Damage)',
    token: 'to Spell Damage',
    defaultRegex: 'to Spell Damage',
    valuePrefix: '\\+?',
    valueSuffix: '% to Spell Damage',
    exampleMin: 20,
  },
  {
    id: 'mf',
    label: '魔法物品获取率 (MF)',
    token: 'Better Chance of Getting Magic Items',
    defaultRegex: 'Better Chance of Getting Magic Items',
    valuePrefix: '\\+?',
    valueSuffix: '% Better Chance of Getting Magic Items',
    exampleMin: 30,
  },
  {
    id: 'gf',
    label: '金币获取量 (GF)',
    token: 'Extra Gold',
    defaultRegex: 'Extra Gold',
    valuePrefix: '\\+?',
    valueSuffix: '% Extra Gold',
    exampleMin: 50,
  },
  {
    id: 'exp',
    label: '经验获取 (Experience Gained)',
    token: 'to Experience Gained',
    defaultRegex: 'to Experience Gained',
    valuePrefix: '\\+?',
    valueSuffix: '% to Experience Gained',
    exampleMin: 5,
  },
];

/**
 * 智能数值下限正则装配算法
 * @param stat 词条定义
 * @param minVal 下限值
 */
export function buildStatThresholdRegex(
  stat: StatOption,
  minVal: number | null | undefined,
): string {
  if (minVal == null || isNaN(minVal) || minVal <= 0) {
    return stat.defaultRegex;
  }

  const prefix = stat.valuePrefix || '';
  const suffix = stat.valueSuffix || '';
  const numRegex = generateNumberGteRegex(minVal);

  return `${prefix}${numRegex}${suffix}`;
}

/**
 * 生成匹配大于等于 target 的整数正则表达式片段
 */
export function generateNumberGteRegex(target: number): string {
  if (target <= 0) return '[0-9]+';
  if (target <= 9) {
    if (target === 1) return '[1-9][0-9]*';
    return `([${target}-9]|[1-9][0-9]+)`;
  }

  const s = String(target);
  const len = s.length;
  const parts: string[] = [];

  // 同等位数，更高数值
  for (let i = 0; i < len; i++) {
    const digit = parseInt(s[i], 10);
    const prefix = s.slice(0, i);

    if (i === len - 1) {
      // 最后一个数位
      if (digit === 9) {
        parts.push(`${prefix}9`);
      } else {
        parts.push(`${prefix}[${digit}-9]`);
      }
    } else {
      if (digit < 9) {
        const nextDigit = digit + 1;
        const remainderDigits = len - 1 - i;
        const higherPart = nextDigit === 9 ? '9' : `[${nextDigit}-9]`;
        parts.push(`${prefix}${higherPart}[0-9]{${remainderDigits}}`);
      }
    }
  }

  // 更多位数的任意正整数
  parts.push(`[1-9][0-9]{${len},}`);

  return `(${parts.join('|')})`;
}

/** 常用过滤规则预设模板 */
export const FILTER_PRESETS: PresetOption[] = [
  {
    name: '💎 镶嵌物-高级符文 (Great Runes)',
    desc: '高亮显示 Ber, Jah, Cham, Zod 及大符文，赤红警示、播放 sound1、小地图红十字标记及通知',
    rule: {
      namePattern: '^(Ber|Jah|Cham|Zod|Taha|Ghal|Qor|Suhe|Kra|Krys|Auhe|Shaad)$',
      color: 'red',
      action: 'show',
      sound: 'sound1',
      notify: true,
      map: true,
    },
  },
  {
    name: '⭐ 品级 [神圣] 暗金装备 (Sacred Unique)',
    desc: '所有神圣暗金 SU/SSU/SSSU 强力装备，黄金色标记、显示 Roll 值词条详情、播放 sound2、地图红十字与通知',
    rule: {
      quality: 'unique',
      tier: 'sacred',
      color: 'gold',
      action: 'show',
      sound: 'sound2',
      notify: true,
      stat: true,
      map: true,
    },
  },
  {
    name: '🛡️ 品级 [神圣] 毕业底材 (无形 0/6 镶孔)',
    desc: '神圣品级普通无形盔甲/武器底材，亮绿标识，小地图绘制标记',
    rule: {
      tier: 'sacred',
      quality: 'normal',
      eth: true,
      color: 'lime',
      action: 'show',
      sound: 'sound3',
      map: true,
    },
  },
  {
    name: '🧿 极品黄装护符 (+2 所有技能)',
    desc: '筛选带有 +2 以上所有技能的黄装护符，粉色高亮、提示通知并在卡片输出词条详情',
    rule: {
      namePattern: 'Amulet$',
      quality: 'rare',
      stats: [{ statId: 'all_skills', minValue: 2 }],
      color: 'pink',
      action: 'show',
      notify: true,
      stat: true,
      map: true,
    },
  },
  {
    name: '🧪 常用高价值杂项 (奥术水晶/遗物/印章/回环)',
    desc: '自动抓取奥术水晶、遗物、印章、回环、完美宝石等核心材料，橙色高亮与小地图标记',
    rule: {
      namePattern: '^(Arcane Crystal|Relic|Signet|Cycle|Perfect .*)$',
      color: 'orange',
      action: 'show',
      sound: 'sound4',
      map: true,
    },
  },
  {
    name: '🚫 隐藏低劣与普通低阶装备 (垃圾过滤)',
    desc: '强制在地面隐藏所有低劣的、超强的或品级 1~4 的普通装备',
    rule: {
      quality: 'low',
      action: 'hide',
    },
  },
];
