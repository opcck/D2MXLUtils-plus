pub(super) const NEW_PROFILE_TEMPLATE: &str = r#"# ==================== 垃圾物品隐藏 ====================

"Gold" hide

# 低阶底模物品 (神圣以下级别)
1 2 3 4 low normal superior rare hide
magic hide

sacred low normal superior magic hide

# 提示神圣无形装备
sacred superior eth notify

# 药水类
".*Healing Potion$" hide
".*Mana Potion$" hide

# ==================== 掉落通报提醒 ====================

# 宝石类
"Onyx|Ruby|Topaz|Diamond|Rainbow Stone|Sapphire|Emerald|Amber|Bloodstone|Amethyst|Skull|Turquoise" hide
"^Perfect" show

# 首饰与珠宝
"Jewel|Quiver" rare notify

"Amulet$" rare {[3-9] to All Skills} stat notify
"Ring$" rare {[1-2] to All Skills} stat notify

# 暗金与套装
unique notify
set notify map

# 神圣暗金
sacred unique notify map

# 天使品质 (Angelic)
angelic notify

# 匠品装备 (Mastercrafted)
master show notify map purple sound1

# 符文类
#"^(El|Eld|Tir|Nef|Eth|Ith|Tal|Ral|Ort|Thul|Amn|Sol|Shael|Dol|Hel|Io|Lum|Ko|Fal|Lem|Pul|Um|Mal|Ist|Gul|Vex|Ohm|Lo|Sur|Ber|Jah|Cham|Zod) Rune$"
"^(Ber|Jah|Cham|Zod) Rune$" notify

[notify map sound3] {
  "Great Rune"
  "Enchanted Rune"
  "Elemental Rune"
  "Container" purple
  "Runestone|Essence$" red
}

# 消耗品
[notify] {
  "Mystic Orb"
  "Arcane (Shard|Crystal|Cluster)"
  "Heavenly|Crate"
  "Shrine \(10"
  "Vessel"
}

# 精华、圣物、奥术材料、附魔卷轴
[notify map sound2] {
  "Essence"
  "Corrupted (Shard|Crystal|Cluster)"
  "Enchant Scroll"
}

# 炼金与锻造试剂
[notify] {
  "Enchanting"
  "Mystic Dye"
  "Treasure"
  "Item Design"
}

# 油剂与特殊消耗品
[notify map sound2] {
  "Oil of Augmentation"
  "Oil of Conjuration"
  "Oil of Greater Luck"
  "Oil of Intensity"
  "Belladonna Extract"
  "Heavenly Soul"
}

# 任务物品
[notify orange map] {
  "Ring of the Five"
  "Sigil$"
  "Tome of Possession"
  "Tenet"
  "Book of Cain"
  "Positronic Brain"
}

"Quest Item|Cube Reagent" notify orange map

"Riftstone" red notify map
"Relic" red notify map sound3

# 战利品、雕像、印记
[notify map] {
  "Trophy"
  "Occult Effigy"
  "Emblem of"
}

# 轮回石 (Cycle)
[notify] {
  "Cycle"
  "Medium Cycle" sound1
  "Large Cycle" sound2
  "Golden Cycle" red sound3 map
}

# 契印 (Signet)
[notify orange map] {
  "Signet of Learning"
}

# 护符 (Charms)
[stat green notify map] {
  "Zakarum's Ear|Visions of Akarat|Bone Chimes|Spirit Trance Herb|Soul of Kabraxis|Fool's Gold"
  "Sunstone of the Twin Seas|The Butcher's Tooth|Optical Detector|Laser Focus Crystal|Scroll of Kings|Moon of the Spider|Horazon's Focus|Six Angel Bag"
  "Sacred Worldstone Key|The Black Road|Azmodan's Heart|Hammer of the Taan Judges|Sunstone of the Gods|Spirit of Creation|Idol of Vanity|Silver Seal of Ureh"
  "Crystalline Flame Medallion|Legacy of Blood|Weather Control|Demonsbane|Umbaru Treasure|Xazax's Illusion|The Ancient Repositories|The Sleep|Dragon Claw|Neutrality Pact"
  "Eternal Bone Pile|Corrupted Wormhole|Cold Fusion Schematics|Lylia's Curse|Astrogha's Venom Stinger|The Glorious Book of Median|Books of Kalan|Vial of Elder Blood"
}

# 词条规则筛选组合示例
# [rare angelic stat notify] {
#   {focus} {enemy fire}
#   {focus} {speeds}
#   {speeds} {enemy fire}
#   "Light Plated" {speeds} {focus}
#   {Frozen Soul}
# }
#
# "Amulet" rare {[3-9] to All Skills} {focus} {enemy fire} stat notify
# "Arrow Quiver" {druid} rare stat notify
"#;
